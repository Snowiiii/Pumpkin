use crate::item::Item;

impl Item {
    pub fn is_helmet(&self) -> bool {
        [
            // Leather
            856, // Netherite
            876, // Turtle helmet
            794, // Chainmail
            860, // Diamond
            868, // Gold
            872, // Iron
            864,
        ]
        .contains(&self.item_id)
    }

    pub fn is_chestplate(&self) -> bool {
        [
            // Leather
            857, // Netherite
            877, // Chainmail
            861, // Diamond
            869, // Gold
            873, // Iron
            865, // Elytra
            773,
        ]
        .contains(&self.item_id)
    }

    pub fn is_leggings(&self) -> bool {
        [
            // Leather
            858, // Netherite
            878, // Chainmail
            862, // Diamond
            870, // Gold
            874, // Iron
            866,
        ]
        .contains(&self.item_id)
    }

    pub fn is_boots(&self) -> bool {
        [
            // Leather
            859, // Netherite
            879, // Chainmail
            863, // Diamond
            871, // Gold
            875, // Iron
            867,
        ]
        .contains(&self.item_id)
    }
}
