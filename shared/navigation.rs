//! Canonical navigation metadata shared by the interactive portfolio and the
//! static article generator. Keeping labels and destinations here prevents the
//! two independently-rendered surfaces from silently drifting.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavItem {
    pub label: &'static str,
    pub href: &'static str,
}

pub const PRIMARY_NAV: [NavItem; 5] = [
    NavItem {
        label: "Work",
        href: "#work",
    },
    NavItem {
        label: "Stack",
        href: "#capabilities",
    },
    NavItem {
        label: "Experience",
        href: "#experience",
    },
    NavItem {
        label: "Contact",
        href: "#contact",
    },
    NavItem {
        label: "Writing",
        href: "/articles/",
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::PRIMARY_NAV;

    /// Duplicate destinations make keyboard and mobile navigation disagree in
    /// subtle ways, while a relative Writing URL breaks on generated article
    /// routes. Pin both properties at the shared-data boundary.
    #[test]
    fn primary_navigation_has_unique_stable_destinations() {
        let destinations: HashSet<_> = PRIMARY_NAV.iter().map(|item| item.href).collect();

        assert_eq!(destinations.len(), PRIMARY_NAV.len());
        assert_eq!(PRIMARY_NAV.last().map(|item| item.href), Some("/articles/"));
        assert!(
            PRIMARY_NAV[..PRIMARY_NAV.len() - 1]
                .iter()
                .all(|item| item.href.starts_with('#'))
        );
    }
}
