use crate::{
    economy::{
        industries::Industry,
    },
    simulation::{SimFlo, SimInt},
};

#[derive(Debug)]
pub struct UnitProductionCost {
    pub energy: SimInt,
    pub labor: SimFlo,
    pub raw_materials: SimFlo,
    pub equipment_maintenance: SimFlo,
    pub packaging: SimFlo,
}

#[derive(Debug)]
pub struct ProductDemandTimeline {
    pub inc_quarter: SimInt,
    pub dec_quarter: SimInt,
    pub dec_half: SimInt,
    pub dec_three_quarters: SimInt,
    pub deadline: SimInt,
}

#[derive(Debug)]
pub struct ProductDemandInfo {
    pub min_percentage: SimFlo,
    pub max_percentage: SimFlo,
    pub unit_per_percent: SimInt,
    pub demand_timeline: ProductDemandTimeline,
}

#[derive(Debug)]
pub struct Product {
    pub name: &'static str,
    pub description: &'static str,
    pub unit_production_cost: UnitProductionCost,
    pub units_per_minute: SimInt,
    pub rnd_cost: SimFlo,
    pub industry: Industry,
    pub demand_info: ProductDemandInfo,
}
impl PartialEq for Product {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl PartialEq<&Product> for Product {
    fn eq(&self, other: &&Product) -> bool {
        self.name == other.name
    }
}

impl Product {
    pub const INTEGRATED_CIRCUIT: Self = Self {
        name: "Integrated Circuit",
        description: "A keep it simple and stupid (KISS) type of chip with a single purpose and sleigh-of-hand capabilities.",
        unit_production_cost: UnitProductionCost {
            energy: 3,
            labor: 11.4,
            raw_materials: 3.1,
            equipment_maintenance: 0.4,
            packaging: 0.8,
        },
        units_per_minute: 5,
        rnd_cost: 7569.56,
        industry: Industry::SEMICONDUCTORS,
        demand_info: ProductDemandInfo {
            min_percentage: 45.0,
            max_percentage: 80.0,
            unit_per_percent: 25,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 2,
                dec_half: 4,
                dec_three_quarters: 8,
                deadline: 16,
            }
        }
    };
    pub const MICROCHIP: Self = Self {
        name: "Microchip",
        description: "A complex electrical organism that knows only two numbers but is so fast that you ignore it's illiteracy in math.",
        unit_production_cost: UnitProductionCost {
            energy: 12,
            labor: 26.45,
            raw_materials: 9.14,
            equipment_maintenance: 6.78,
            packaging: 9.23,
        },
        units_per_minute: 1,
        rnd_cost: 69376.12,
        industry: Industry::SEMICONDUCTORS,
        demand_info: ProductDemandInfo {
            min_percentage: 65.0,
            max_percentage: 95.0,
            unit_per_percent: 10,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 3,
                dec_half: 6,
                dec_three_quarters: 8,
                deadline: 12,
            }
        }
    };
    pub const SAAS: Self = Self {
        name: "SaaS",
        description: "Software as a Sauce can deliver what you need or don't need but think you need right to your door or to your face, eyes and ears.",
        unit_production_cost: UnitProductionCost {
            energy: 1,
            labor: 18.10,
            raw_materials: 0.4,
            equipment_maintenance: 1.63,
            packaging: 0.2,
        },
        units_per_minute: 10,
        rnd_cost: 3670.45,
        industry: Industry::SOFTWARE,
        demand_info: ProductDemandInfo {
            min_percentage: 15.0,
            max_percentage: 75.0,
            unit_per_percent: 32,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 3,
                dec_half: 5,
                dec_three_quarters: 7,
                deadline: 9,
            }
        }
    };
    pub const COMPUTER_VIRUS: Self = Self {
        name: "Computer Virus",
        description: "Used for industrial espionage and some shady government operations that don't benefit you as a normie at all but who cares.",
        unit_production_cost: UnitProductionCost {
            energy: 3,
            labor: 44.10,
            raw_materials: 0.4,
            equipment_maintenance: 2.12,
            packaging: 0.1,
        },
        units_per_minute: 1,
        rnd_cost: 87450.23,
        industry: Industry::SOFTWARE,
        demand_info: ProductDemandInfo {
            min_percentage: 60.0,
            max_percentage: 70.0,
            unit_per_percent: 3,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 5,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 16,
            }
        }
    };
    pub const PAPER: Self = Self {
        name: "Paper",
        description: "People use our paper to buy things, not your paper or somebody else's paper because it's a sin.",
        unit_production_cost: UnitProductionCost {
            energy: 5,
            labor: 10.45,
            raw_materials: 1.9,
            equipment_maintenance: 2.31,
            packaging: 4.5,
        },
        units_per_minute: 20,
        rnd_cost: 9100.49,
        industry: Industry::BANK,
        demand_info: ProductDemandInfo {
            min_percentage: 90.0,
            max_percentage: 100.0,
            unit_per_percent: 42,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 4,
                dec_half: 6,
                dec_three_quarters: 8,
                deadline: 10,
            }
        }
    };
    pub const DEBT: Self = Self {
        name: "Debt",
        description: "The most valuable commodity in the universe. Converts humans to easily controllable subjects.",
        unit_production_cost: UnitProductionCost {
            energy: 3,
            labor: 89.54,
            raw_materials: 0.2,
            equipment_maintenance: 0.23,
            packaging: 1.2,
        },
        units_per_minute: 3,
        rnd_cost: 105234.8,
        industry: Industry::BANK,
        demand_info: ProductDemandInfo {
            min_percentage: 70.0,
            max_percentage: 100.0,
            unit_per_percent: 21,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 4,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 20,
            }
        }
    };
    pub const SKIN_CLEANER: Self = Self {
        name: "Skin Cleaner",
        description: "Chemicals clean the skin because they are toxic and kill both beneficial or harmful bacteria. So look, it's clean.",
        unit_production_cost: UnitProductionCost {
            energy: 5,
            labor: 6.43,
            raw_materials: 0.6,
            equipment_maintenance: 0.33,
            packaging: 22.32,
        },
        units_per_minute: 20,
        rnd_cost: 3467.76,
        industry: Industry::COSMETICS,
        demand_info: ProductDemandInfo {
            min_percentage: 15.0,
            max_percentage: 50.0,
            unit_per_percent: 32,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 4,
                dec_half: 6,
                dec_three_quarters: 8,
                deadline: 12,
            }
        }
    };
    pub const ELIXIR_OF_YOUTH: Self = Self {
        name: "Elixir of Youth",
        description: "Even if you die today, don't you want to look gorgeous in your coffin?",
        unit_production_cost: UnitProductionCost {
            energy: 4,
            labor: 8.34,
            raw_materials: 0.3,
            equipment_maintenance: 4.12,
            packaging: 48.54,
        },
        units_per_minute: 3,
        rnd_cost: 83456.71,
        industry: Industry::COSMETICS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 75.0,
            unit_per_percent: 17,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 2,
                dec_half: 3,
                dec_three_quarters: 4,
                deadline: 5,
            }
        }
    };
    pub const UNGUIDED_ROCKET: Self = Self {
        name: "Unguided Rocket",
        description: "You can spray and pray with this and tell your superiors you did something for the country.",
        unit_production_cost: UnitProductionCost {
            energy: 12,
            labor: 17.42,
            raw_materials: 4.53,
            equipment_maintenance: 6.34,
            packaging: 40.4,
        },
        units_per_minute: 5,
        rnd_cost: 5698.34,
        industry: Industry::MISSILES,
        demand_info: ProductDemandInfo {
            min_percentage: 50.0,
            max_percentage: 85.0,
            unit_per_percent: 40,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 4,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 14,
            }
        }
    };
    pub const GUIDED_MISSILE: Self = Self {
        name: "Guided Missile",
        description: "You can spray and pray all right but sometimes there are high value targets to hit. Don't hit all high value targets at once though. Leave some for hitting later so there's always something to hit at any given time.",
        unit_production_cost: UnitProductionCost {
            energy: 64,
            labor: 803.24,
            raw_materials: 84.23,
            equipment_maintenance: 17.34,
            packaging: 642.39,
        },
        units_per_minute: 1,
        rnd_cost: 253875.5,
        industry: Industry::MISSILES,
        demand_info: ProductDemandInfo {
            min_percentage: 40.0,
            max_percentage: 100.0,
            unit_per_percent: 5,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 3,
                dec_quarter: 14,
                dec_half: 20,
                dec_three_quarters: 22,
                deadline: 23,
            }
        }
    };
    pub const AMMO: Self = Self {
        name: "Ammo",
        description: "These brass cylinders with lead and nitrocellulose in them get spent so quickly that we can't keep-up with the demand. Too bad the consumers themselves may also get spent spending them.",
        unit_production_cost: UnitProductionCost {
            energy: 24,
            labor: 83.72,
            raw_materials: 14.78,
            equipment_maintenance: 1.48,
            packaging: 0.3,
        },
        units_per_minute: 40,
        rnd_cost: 1874.32,
        industry: Industry::ARMS,
        demand_info: ProductDemandInfo {
            min_percentage: 90.0,
            max_percentage: 100.0,
            unit_per_percent: 100,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 16,
                dec_half: 32,
                dec_three_quarters: 64,
                deadline: 128,
            }
        }
    };
    pub const SEMI_AUTO: Self = Self {
        name: "Semi-Auto",
        description: "All armies around the world, government, private or the mafia love our guns. They sleep with them. They oil their inner tubes for the smooth operation of.. ammo.",
        unit_production_cost: UnitProductionCost {
            energy: 640,
            labor: 78.19,
            raw_materials: 75.34,
            equipment_maintenance: 9.35,
            packaging: 13.12,
        },
        units_per_minute: 6,
        rnd_cost: 47849.28,
        industry: Industry::ARMS,
        demand_info: ProductDemandInfo {
            min_percentage: 60.0,
            max_percentage: 85.0,
            unit_per_percent: 24,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 16,
                dec_three_quarters: 18,
                deadline: 20,
            }
        }
    };
    pub const SUGAR_DRINK: Self = Self {
        name: "Sugar Drink",
        description: "Though causes obesity and heart disease, noone imposes high taxes on these. We're so lucky. Hahahah :)",
        unit_production_cost: UnitProductionCost {
            energy: 3,
            labor: 1.5,
            raw_materials: 1.43,
            equipment_maintenance: 0.34,
            packaging: 2.78,
        },
        units_per_minute: 24,
        rnd_cost: 538.29,
        industry: Industry::PROCESSED_FOODS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 70.0,
            unit_per_percent: 100,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 16,
                dec_three_quarters: 32,
                deadline: 40,
            }
        }
    };
    pub const SYNTHETIC_MEAT: Self = Self {
        name: "Synthetic Meat",
        description: "It's eighty five percent vegan. But that's not the only selling point.",
        unit_production_cost: UnitProductionCost {
            energy: 8,
            labor: 79.43,
            raw_materials: 920.54,
            equipment_maintenance: 18.34,
            packaging: 12.55,
        },
        units_per_minute: 10,
        rnd_cost: 136592.0,
        industry: Industry::PROCESSED_FOODS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 70.0,
            unit_per_percent: 75,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 8,
                dec_half: 16,
                dec_three_quarters: 24,
                deadline: 28,
            }
        }
    };
    pub const PREGNANCY_TEST: Self = Self {
        name: "Pregnancy Test",
        description: "Getting pregnant is expensive, pregnancy tests are cheap. Do the math.",
        unit_production_cost: UnitProductionCost {
            energy: 6,
            labor: 138.41,
            raw_materials: 256.68,
            equipment_maintenance: 6.1,
            packaging: 13.58,
        },
        units_per_minute: 15,
        rnd_cost: 8735.39,
        industry: Industry::PHARMACEUTICALS,
        demand_info: ProductDemandInfo {
            min_percentage: 75.0,
            max_percentage: 100.0,
            unit_per_percent: 30,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 12,
                dec_half: 24,
                dec_three_quarters: 48,
                deadline: 128,
            }
        }
    };
    pub const BIRTH_CONTROL_PILL: Self = Self {
        name: "Birth Control Pill",
        description: "Some religions don't like this product, but we bought the majority of them long time ago.",
        unit_production_cost: UnitProductionCost {
            energy: 3,
            labor: 674.23,
            raw_materials: 192.24,
            equipment_maintenance: 16.11,
            packaging: 26.34,
        },
        units_per_minute: 8,
        rnd_cost: 826658.3,
        industry: Industry::PHARMACEUTICALS,
        demand_info: ProductDemandInfo {
            min_percentage: 55.0,
            max_percentage: 82.0,
            unit_per_percent: 13,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 6,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 32,
            }
        }
    };
    pub const CHATBOT_TOKENS: Self = Self {
        name: "Chatbot Tokens",
        description: "Now you can move R&D and ops to overseas and pretend using chatbots instead to boost your stock prices.",
        unit_production_cost: UnitProductionCost {
            energy: 12,
            labor: 421.60,
            raw_materials: 29.49,
            equipment_maintenance: 3.32,
            packaging: 0.7,
        },
        units_per_minute: 26,
        rnd_cost: 8285.2,
        industry: Industry::E_YAY,
        demand_info: ProductDemandInfo {
            min_percentage: 12.0,
            max_percentage: 45.0,
            unit_per_percent: 110,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 4,
                dec_half: 6,
                dec_three_quarters: 8,
                deadline: 10,
            }
        }
    };
    pub const ASSISTANT_INTRUDER: Self = Self {
        name: "Assistant Intruder",
        description: "An E-YAY assistant that can be used defensively and offensively. We profit either way.",
        unit_production_cost: UnitProductionCost {
            energy: 88,
            labor: 1920.54,
            raw_materials: 4110.32,
            equipment_maintenance: 52.4,
            packaging: 894.31,
        },
        units_per_minute: 4,
        rnd_cost: 748293.22,
        industry: Industry::E_YAY,
        demand_info: ProductDemandInfo {
            min_percentage: 45.0,
            max_percentage: 70.0,
            unit_per_percent: 12,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 10,
                dec_three_quarters: 12,
                deadline: 14,
            }
        }
    };
    pub const GRADUATE: Self = Self {
        name: "Graduate",
        description: "Students have a good time in our university and realize it wasn't free after they graduate. Better than credit cards.",
        unit_production_cost: UnitProductionCost {
            energy: 66,
            labor: 3256.43,
            raw_materials: 1246.33,
            equipment_maintenance: 21.98,
            packaging: 35.6,
        },
        units_per_minute: 5,
        rnd_cost: 102.30,
        industry: Industry::UNIVERSITY,
        demand_info: ProductDemandInfo {
            min_percentage: 50.0,
            max_percentage: 80.0,
            unit_per_percent: 25,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 6,
                dec_half: 10,
                dec_three_quarters: 16,
                deadline: 24,
            }
        }
    };
    pub const STUDENT_DEBT: Self = Self {
        name: "Student Debt",
        description: "Thinking, innovating, job creating human-machines with shitloads of debt. What better tool to advance our evolving revolving economy!",
        unit_production_cost: UnitProductionCost {
            energy: 11,
            labor: 320.45,
            raw_materials: 321.21,
            equipment_maintenance: 18.23,
            packaging: 0.0,
        },
        units_per_minute: 1,
        rnd_cost: 984_054.7,
        industry: Industry::UNIVERSITY,
        demand_info: ProductDemandInfo {
            min_percentage: 20.0,
            max_percentage: 60.0,
            unit_per_percent: 22,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 4,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 16,
            }
        }
    };
}

impl Product {
    pub fn by_industry(industry: &Industry) -> Vec<&Self> {
        Vec::from(PRODUCTS.iter().filter(|&product| &product.industry == industry).collect::<Vec<_>>())
    }

    pub fn get_unit_cost_excl_energy(&self) -> SimFlo {
        let unit_pc = &self.unit_production_cost;

        (unit_pc.packaging + unit_pc.labor + unit_pc.raw_materials + unit_pc.equipment_maintenance).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductStock {
    pub product: &'static Product,
    pub units: SimInt,
    pub unit_production_cost: SimFlo,
}

pub const PRODUCTS: &[Product] = &[
    Product::INTEGRATED_CIRCUIT,
    Product::MICROCHIP,
    Product::SAAS,
    Product::COMPUTER_VIRUS,
    Product::PAPER,
    Product::DEBT,
    Product::SKIN_CLEANER,
    Product::ELIXIR_OF_YOUTH,
    Product::UNGUIDED_ROCKET,
    Product::GUIDED_MISSILE,
    Product::AMMO,
    Product::SEMI_AUTO,
    Product::SUGAR_DRINK,
    Product::SYNTHETIC_MEAT,
    Product::PREGNANCY_TEST,
    Product::BIRTH_CONTROL_PILL,
    Product::CHATBOT_TOKENS,
    Product::ASSISTANT_INTRUDER,
    Product::GRADUATE,
    Product::STUDENT_DEBT,
];