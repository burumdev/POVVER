use crate::{
    economy::{
        industries::Industry,
        economy_types::{Money, EnergyUnit},
    },
    simulation::{SimFlo, SimInt},
};

#[derive(Debug)]
pub struct UnitProductionCost {
    pub energy: EnergyUnit,
    pub labor: Money,
    pub raw_materials: Money,
    pub equipment_maintenance: Money,
    pub packaging: Money,
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
    pub rnd_cost: Money,
    pub industry: Industry,
    pub demand_info: ProductDemandInfo,
}
impl PartialEq for Product {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Product {
    pub const INTEGRATED_CIRCUIT: Self = Self {
        name: "Integrated Circuit",
        description: "A keep it simple and stupid (KISS) type of chip with a single purpose and sleigh-of-hand capabilities.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(6),
            labor: Money::new(11.4),
            raw_materials: Money::new(3.1),
            equipment_maintenance: Money::new(8.85),
            packaging: Money::new(0.8),
        },
        rnd_cost: Money::new(7569.56),
        industry: Industry::SEMICONDUCTORS,
        demand_info: ProductDemandInfo {
            min_percentage: 45.0,
            max_percentage: 80.0,
            unit_per_percent: 100,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 6,
                dec_half: 12,
                dec_three_quarters: 16,
                deadline: 24,
            }
        }
    };
    pub const MICROCHIP: Self = Self {
        name: "Microchip",
        description: "A complex electrical organism that knows only two numbers but is so fast that you ignore it's illiteracy in math.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(15),
            labor: Money::new(26.45),
            raw_materials: Money::new(9.14),
            equipment_maintenance: Money::new(62.78),
            packaging: Money::new(9.23),
        },
        rnd_cost: Money::new(69376.12),
        industry: Industry::SEMICONDUCTORS,
        demand_info: ProductDemandInfo {
            min_percentage: 65.0,
            max_percentage: 95.0,
            unit_per_percent: 50,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 4,
                dec_half: 8,
                dec_three_quarters: 12,
                deadline: 16,
            }
        }
    };
    pub const SAAS: Self = Self {
        name: "SaaS",
        description: "Software as a Sauce can deliver what you need or don't need but think you need right to your door or to your face, eyes and ears.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(1),
            labor: Money::new(18.10),
            raw_materials: Money::new(0.4),
            equipment_maintenance: Money::new(11.63),
            packaging: Money::new(0.2),
        },
        rnd_cost: Money::new(3670.45),
        industry: Industry::SOFTWARE,
        demand_info: ProductDemandInfo {
            min_percentage: 15.0,
            max_percentage: 75.0,
            unit_per_percent: 50,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 6,
                dec_half: 8,
                dec_three_quarters: 10,
                deadline: 12,
            }
        }
    };
    pub const COMPUTER_VIRUS: Self = Self {
        name: "Computer Virus",
        description: "Used for industrial espionage and some shady government operations that don't benefit you as a normie at all but who cares.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(3),
            labor: Money::new(44.10),
            raw_materials: Money::new(0.4),
            equipment_maintenance: Money::new(9.12),
            packaging: Money::new(0.1),
        },
        rnd_cost: Money::new(87450.23),
        industry: Industry::SOFTWARE,
        demand_info: ProductDemandInfo {
            min_percentage: 60.0,
            max_percentage: 70.0,
            unit_per_percent: 5,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 10,
                dec_half: 16,
                dec_three_quarters: 24,
                deadline: 32,
            }
        }
    };
    pub const PAPER: Self = Self {
        name: "Paper",
        description: "People use our paper to buy things, not your paper or somebody else's paper because it's a sin.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(5),
            labor: Money::new(10.45),
            raw_materials: Money::new(1.9),
            equipment_maintenance: Money::new(72.31),
            packaging: Money::new(4.5),
        },
        rnd_cost: Money::new(9100.49),
        industry: Industry::BANK,
        demand_info: ProductDemandInfo {
            min_percentage: 90.0,
            max_percentage: 100.0,
            unit_per_percent: 1000,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 1,
                dec_quarter: 16,
                dec_half: 24,
                dec_three_quarters: 32,
                deadline: 64,
            }
        }
    };
    pub const DEBT: Self = Self {
        name: "Debt",
        description: "The most valuable commodity in the universe. Converts humans to easily controllable subjects.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(3),
            labor: Money::new(89.54),
            raw_materials: Money::new(0.2),
            equipment_maintenance: Money::new(8.23),
            packaging: Money::new(1.2),
        },
        rnd_cost: Money::new(105234.8),
        industry: Industry::BANK,
        demand_info: ProductDemandInfo {
            min_percentage: 70.0,
            max_percentage: 100.0,
            unit_per_percent: 175,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 8,
                dec_half: 12,
                dec_three_quarters: 14,
                deadline: 16,
            }
        }
    };
    pub const SKIN_CLEANER: Self = Self {
        name: "Skin Cleaner",
        description: "Chemicals clean the skin because they are toxic and kill both beneficial or harmful bacteria. So look, it's clean.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(7),
            labor: Money::new(6.43),
            raw_materials: Money::new(0.6),
            equipment_maintenance: Money::new(6.23),
            packaging: Money::new(42.32),
        },
        rnd_cost: Money::new(3467.76),
        industry: Industry::COSMETICS,
        demand_info: ProductDemandInfo {
            min_percentage: 10.0,
            max_percentage: 50.0,
            unit_per_percent: 1780,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 8,
                dec_quarter: 10,
                dec_half: 12,
                dec_three_quarters: 14,
                deadline: 16,
            }
        }
    };
    pub const ELIXIR_OF_YOUTH: Self = Self {
        name: "Elixir of Youth",
        description: "Even if you die today, don't you want to look gorgeous in your coffin?",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(4),
            labor: Money::new(8.34),
            raw_materials: Money::new(0.3),
            equipment_maintenance: Money::new(6.12),
            packaging: Money::new(248.54),
        },
        rnd_cost: Money::new(83456.71),
        industry: Industry::COSMETICS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 75.0,
            unit_per_percent: 777,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 12,
                dec_three_quarters: 16,
                deadline: 20,
            }
        }
    };
    pub const UNGUIDED_ROCKET: Self = Self {
        name: "Unguided Rocket",
        description: "You can spray and pray with this and tell your superiors you did something for the country.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(12),
            labor: Money::new(17.42),
            raw_materials: Money::new(4.53),
            equipment_maintenance: Money::new(68.34),
            packaging: Money::new(120.40),
        },
        rnd_cost: Money::new(5698.34),
        industry: Industry::MISSILES,
        demand_info: ProductDemandInfo {
            min_percentage: 50.0,
            max_percentage: 85.0,
            unit_per_percent: 1600,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 2,
                dec_quarter: 10,
                dec_half: 16,
                dec_three_quarters: 24,
                deadline: 64,
            }
        }
    };
    pub const GUIDED_MISSILE: Self = Self {
        name: "Guided Missile",
        description: "You can spray and pray all right but sometimes there are high value targets to hit. Don't hit all high value targets at once though. Leave some for hitting later so there's always something to hit at any given time.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(64),
            labor: Money::new(803.24),
            raw_materials: Money::new(84.23),
            equipment_maintenance: Money::new(174.34),
            packaging: Money::new(642.39),
        },
        rnd_cost: Money::new(253875.5),
        industry: Industry::MISSILES,
        demand_info: ProductDemandInfo {
            min_percentage: 40.0,
            max_percentage: 100.0,
            unit_per_percent: 250,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 3,
                dec_quarter: 14,
                dec_half: 20,
                dec_three_quarters: 32,
                deadline: 64,
            }
        }
    };
    pub const AMMO: Self = Self {
        name: "Ammo",
        description: "These brass cylinders with lead and nitrocellulose in them get spent so quickly that we can't keep-up with the demand. Too bad the consumers themselves may also get spent spending them.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(24),
            labor: Money::new(83.72),
            raw_materials: Money::new(14.78),
            equipment_maintenance: Money::new(376.48),
            packaging: Money::new(0.3),
        },
        rnd_cost: Money::new(1874.32),
        industry: Industry::ARMS,
        demand_info: ProductDemandInfo {
            min_percentage: 90.0,
            max_percentage: 100.0,
            unit_per_percent: 30000,
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
            energy: EnergyUnit::new(640),
            labor: Money::new(784.19),
            raw_materials: Money::new(758.34),
            equipment_maintenance: Money::new(984.35),
            packaging: Money::new(92.12),
        },
        rnd_cost: Money::new(47849.28),
        industry: Industry::ARMS,
        demand_info: ProductDemandInfo {
            min_percentage: 60.0,
            max_percentage: 85.0,
            unit_per_percent: 2200,
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
            energy: EnergyUnit::new(3),
            labor: Money::new(1.5),
            raw_materials: Money::new(17.43),
            equipment_maintenance: Money::new(420.34),
            packaging: Money::new(42.78),
        },
        rnd_cost: Money::new(538.29),
        industry: Industry::PROCESSED_FOODS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 70.0,
            unit_per_percent: 12655,
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
            energy: EnergyUnit::new(8),
            labor: Money::new(79.43),
            raw_materials: Money::new(920.54),
            equipment_maintenance: Money::new(1845.34),
            packaging: Money::new(82.55),
        },
        rnd_cost: Money::new(136592.0),
        industry: Industry::PROCESSED_FOODS,
        demand_info: ProductDemandInfo {
            min_percentage: 30.0,
            max_percentage: 70.0,
            unit_per_percent: 5000,
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
            energy: EnergyUnit::new(6),
            labor: Money::new(138.41),
            raw_materials: Money::new(256.68),
            equipment_maintenance: Money::new(3659.1),
            packaging: Money::new(163.58),
        },
        rnd_cost: Money::new(8735.39),
        industry: Industry::PHARMACEUTICALS,
        demand_info: ProductDemandInfo {
            min_percentage: 75.0,
            max_percentage: 100.0,
            unit_per_percent: 6800,
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
            energy: EnergyUnit::new(3),
            labor: Money::new(674.23),
            raw_materials: Money::new(192.24),
            equipment_maintenance: Money::new(6984.11),
            packaging: Money::new(56.34),
        },
        rnd_cost: Money::new(826658.3),
        industry: Industry::PHARMACEUTICALS,
        demand_info: ProductDemandInfo {
            min_percentage: 55.0,
            max_percentage: 82.0,
            unit_per_percent: 3428,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 3,
                dec_quarter: 12,
                dec_half: 24,
                dec_three_quarters: 48,
                deadline: 64,
            }
        }
    };
    pub const CHATBOT_TOKENS: Self = Self {
        name: "Chatbot Tokens",
        description: "Now you can move R&D and ops to overseas and pretend using chatbots instead to boost your stock prices.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(12),
            labor: Money::new(421.60),
            raw_materials: Money::new(29.49),
            equipment_maintenance: Money::new(3184.32),
            packaging: Money::new(0.7),
        },
        rnd_cost: Money::new(8285.2),
        industry: Industry::E_YAY,
        demand_info: ProductDemandInfo {
            min_percentage: 12.0,
            max_percentage: 45.0,
            unit_per_percent: 128670,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 8,
                dec_quarter: 12,
                dec_half: 14,
                dec_three_quarters: 16,
                deadline: 18,
            }
        }
    };
    pub const ASSISTANT_INTRUDER: Self = Self {
        name: "Assistant Intruder",
        description: "An E-YAY assistant that can be used defensively and offensively. We profit either way.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(88),
            labor: Money::new(1920.54),
            raw_materials: Money::new(4110.32),
            equipment_maintenance: Money::new(52456.4),
            packaging: Money::new(894.31),
        },
        rnd_cost: Money::new(748293.22),
        industry: Industry::E_YAY,
        demand_info: ProductDemandInfo {
            min_percentage: 45.0,
            max_percentage: 70.0,
            unit_per_percent: 100,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 12,
                dec_three_quarters: 16,
                deadline: 24,
            }
        }
    };
    pub const GRADUATE: Self = Self {
        name: "Graduate",
        description: "Students have a good time in our university and realize it wasn't free after they graduate. Better than credit cards.",
        unit_production_cost: UnitProductionCost {
            energy: EnergyUnit::new(66),
            labor: Money::new(3256.43),
            raw_materials: Money::new(1246.33),
            equipment_maintenance: Money::new(2156.98),
            packaging: Money::new(35.6),
        },
        rnd_cost: Money::new(102.30),
        industry: Industry::UNIVERSITY,
        demand_info: ProductDemandInfo {
            min_percentage: 50.0,
            max_percentage: 80.0,
            unit_per_percent: 1500,
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
            energy: EnergyUnit::new(11),
            labor: Money::new(320.45),
            raw_materials: Money::new(321.21),
            equipment_maintenance: Money::new(10.23),
            packaging: Money::new(0.0),
        },
        rnd_cost: Money::new(984_054.7),
        industry: Industry::UNIVERSITY,
        demand_info: ProductDemandInfo {
            min_percentage: 20.0,
            max_percentage: 60.0,
            unit_per_percent: 2500,
            demand_timeline: ProductDemandTimeline {
                inc_quarter: 4,
                dec_quarter: 8,
                dec_half: 12,
                dec_three_quarters: 16,
                deadline: 24,
            }
        }
    };
}

impl Product {
    pub fn by_industry(industry: &Industry) -> Vec<&Self> {
        Vec::from(PRODUCTS.iter().filter(|&product| &product.industry == industry).collect::<Vec<_>>())
    }
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