#[derive(Debug)]
pub struct Industry {
    name: &'static str,
    description: &'static str,
}

impl Industry {
    pub const SEMICONDUCTORS: Self = Self {
        name: "Semiconductors",
        description: "Processes sand and wires to construct calculating glass gadgets.",
    };
    pub const SOFTWARE: Self = Self {
        name: "Software",
        description: "Uses variables and pointers to produce magical binary blobs.",
    };
    pub const BANK: Self = Self {
        name: "Bank",
        description: "Produces means of exchange and debt, debt and more debt.",
    };
    pub const COSMETICS: Self = Self {
        name: "Cosmetics",
        description: "Converts toxic chemicals to mildly toxic ones that don't burn your skin in relatively low concentrations.",
    };
    pub const MISSILES: Self = Self {
        name: "Missiles",
        description: "Produces self-propelling bombs with a high kill rate.",
    };
    pub const ARMS: Self = Self {
        name: "Arms",
        description: "Produces metal shooting tools that sell exceptionally well when crime rates are high.",
    };
    pub const PROCESSED_FOODS: Self = Self {
        name: "Processed Foods",
        description: "Manufactures foodstuffs with low nutrition value and very high shelf lives.",
    };
    pub const PHARMACEUTICALS: Self = Self {
        name: "Pharmaceuticals",
        description: "Insulin, sulfonylureas: large doses of salicylates have a hypoglycemic action and may enhance the effect of oral hypoglycemic agents.",
    };
    pub const E_YAY: Self = Self {
        name: "E-YAY!",
        description: "Electronic YAY! Produces chat bots with unlimited access to copyrighted data and compute repetition."
    };
    pub const UNIVERSITY: Self = Self {
        name: "University",
        description: "A conglomeration that sells several years of fun time and treats to students indebting them in the process."
    };
}

pub const INDUSTRIES: &[Industry] = &[
    Industry::SEMICONDUCTORS,
    Industry::SOFTWARE,
    Industry::BANK,
    Industry::COSMETICS,
    Industry::MISSILES,
    Industry::ARMS,
    Industry::PROCESSED_FOODS,
    Industry::PHARMACEUTICALS,
    Industry::E_YAY,
    Industry::UNIVERSITY,
];
