// Choose the band padding by measurement: the gate has to admit every
// published article and reject prose written with no knowledge of the voice.
// Throwaway calibration aid; the chosen value is recorded in lib.rs.
//
// usage: sweep <dir of generic-prose .txt files>
fn main() {
    let generic_dir = std::env::args().nth(1);
    let mut paths: Vec<_> = std::fs::read_dir("content/articles")
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    paths.sort();
    let metrics: Vec<_> = paths
        .iter()
        .map(|p| voice::measure(&std::fs::read_to_string(p).unwrap()))
        .collect();

    let generic: Vec<_> = generic_dir
        .iter()
        .flat_map(|d| {
            std::fs::read_dir(d)
                .unwrap()
                .filter_map(|e| e.ok().map(|e| e.path()))
        })
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .map(|p| voice::measure(&std::fs::read_to_string(p).unwrap()))
        .collect();

    for pad in [0.0f64, 0.25, 0.5, 0.75, 1.0, 1.5] {
        // His own articles, leave-one-out: the band a real check would use.
        let mut own_fail = 0;
        for (i, m) in metrics.iter().enumerate() {
            let bad = voice::CHECKED
                .iter()
                .filter(|(_, get, _)| {
                    let others: Vec<f64> = metrics
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i)
                        .map(|(_, mm)| get(mm))
                        .collect();
                    let b = voice::band_with_pad(&others, pad);
                    let v = get(m);
                    v < b.low || v > b.high
                })
                .count();
            if bad > 0 {
                own_fail += 1;
            }
        }
        // Generic prose, against the full corpus band.
        let mut generic_fail = 0;
        let mut generic_measures = 0;
        for m in &generic {
            let bad = voice::CHECKED
                .iter()
                .filter(|(_, get, _)| {
                    let b = voice::band_with_pad(
                        &metrics.iter().map(|mm| get(mm)).collect::<Vec<_>>(),
                        pad,
                    );
                    let v = get(m);
                    v < b.low || v > b.high
                })
                .count();
            if bad > 0 {
                generic_fail += 1;
            }
            generic_measures += bad;
        }
        println!(
            "pad {pad:>4} sd -> own articles failing: {own_fail:2}/{}   generic failing: {generic_fail}/{} ({generic_measures} measurements)",
            metrics.len(),
            generic.len()
        );
    }
}
