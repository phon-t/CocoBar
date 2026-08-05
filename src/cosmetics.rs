pub(crate) struct CosmeticItem {
    pub(crate) name: &'static str,
    pub(crate) bytes: &'static [u8],
}

pub(crate) const BELL_ITEMS: &[CosmeticItem] = &[
    CosmeticItem {
        name: "Black Bell",
        bytes: include_bytes!("../assets/Cosmetics/bell/bellblack.png"),
    },
    CosmeticItem {
        name: "Red Bell",
        bytes: include_bytes!("../assets/Cosmetics/bell/bellred.png"),
    },
];

pub(crate) const SCARF_ITEMS: &[CosmeticItem] = &[
    CosmeticItem {
        name: "Black Scarf",
        bytes: include_bytes!("../assets/Cosmetics/Scarf/scarfblack.png"),
    },
    CosmeticItem {
        name: "Red Scarf",
        bytes: include_bytes!("../assets/Cosmetics/Scarf/scarfred.png"),
    },
    CosmeticItem {
        name: "Blue Scarf",
        bytes: include_bytes!("../assets/Cosmetics/Scarf/scarfblue.png"),
    },
    CosmeticItem {
        name: "Pink Scarf",
        bytes: include_bytes!("../assets/Cosmetics/Scarf/scarfpink.png"),
    },
    CosmeticItem {
        name: "White Scarf",
        bytes: include_bytes!("../assets/Cosmetics/Scarf/scarfwhite.png"),
    },
];

pub(crate) const TIE_ITEMS: &[CosmeticItem] = &[
    CosmeticItem {
        name: "Blue Tie",
        bytes: include_bytes!("../assets/Cosmetics/tie/tieblue.png"),
    },
    CosmeticItem {
        name: "Orange Tie",
        bytes: include_bytes!("../assets/Cosmetics/tie/tieorange.png"),
    },
    CosmeticItem {
        name: "Red Tie",
        bytes: include_bytes!("../assets/Cosmetics/tie/tiered.png"),
    },
];