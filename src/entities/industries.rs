use crate::simulation::SimInt;

pub struct Industry {
    id: SimInt,
    name: &'static str,
    description: &'static str,
}

pub const INDUSTRIES: &[Industry] = &[
    Industry {
        id: 1,
        name: "Semiconductors",
        description: "Processes sand and wires to construct calculating gadgets.",
    },
    Industry {
        id: 2,
        name: "Software",
        description: "Uses variables and pointers to produce magical binary blobs.",
    },
    Industry {
        id: 3,
        name: "Bank",
        description: "Produces means of exchange and debt.",
    },
    Industry {
        id: 4,
        name: "Cosmetics",
        description: "Converts toxic chemicals to mildly toxic beauty substances.",
    },
    Industry {
        id: 5,
        name: "Missiles",
        description: "Produces self-propelling bombs with a high kill rate.",
    },
    Industry {
        id: 6,
        name: "Arms",
        description: "Produces metal tools that sell exceptionally well with high crime rates.",
    },
    Industry {
        id: 7,
        name: "Processed Foods",
        description: "Manufactures foodstuffs with low nutrition value and very high shelf lives.",
    },
    Industry {
        id: 8,
        name: "Pharmaceuticals",
        description: "Insulin, sulfonylureas: large doses of salicylates have a hypoglycemic action and may enhance the effect of oral hypoglycemic agents.",
    },
    Industry {
        id: 9,
        name: "E-YAY!",
        description: "Electronic YAY! produces chat bots with unlimited access to copyrighted data and compute repetition."
    },
    Industry {
        id: 10,
        name: "University",
        description: "A conglomeration of academia elites that sell several years of fun time and treats to students, indebting them."
    }
];