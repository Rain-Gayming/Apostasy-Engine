use std::fmt::Debug;

use apostasy_macros::Component;

#[derive(Component, Default, Clone, Debug)]
pub struct Container {
    pub items: Vec<ContainerItem>,
}

impl Container {
    pub fn deserialize(&mut self, _value: &serde_yaml::Value) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn add_item(&mut self, item: ContainerItem) -> &mut Self {
        self.items.push(item);
        self
    }
}

#[derive(Clone)]
pub struct ContainerItem {
    pub item: String,
    pub amount: u32,
}

impl Debug for ContainerItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContainerItem")
            .field("Item", &self.item)
            .field("Amount", &self.amount)
            .finish()
    }
}
