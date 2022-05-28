use crate::prelude::*;

/// A complete, witnessed leaf of a tree.
#[derive(Clone, Copy, Derivative, Serialize, Deserialize)]
#[derivative(Debug = "transparent")]
pub struct Leaf<Item>(pub(in super::super) Item);

impl<Item> Leaf<Item> {
    /// Create a new complete leaf from the item stored in the tree.
    pub fn new(item: Item) -> Self {
        Self(item)
    }
}

impl<Item: GetHash> GetHash for Leaf<Item> {
    #[inline]
    fn hash(&self) -> Hash {
        self.0.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.0.cached_hash()
    }
}

impl<Item: Height> Height for Leaf<Item> {
    type Height = Item::Height;
}

impl<Item: Complete> Complete for Leaf<Item> {
    type Focus = frontier::Leaf<<Item as Complete>::Focus>;
}

impl<Item: Witness> Witness for Leaf<Item> {
    #[inline]
    fn witness(&self, index: impl Into<u64>) -> Option<(AuthPath<Self>, Hash)> {
        self.0.witness(index)
    }
}

impl<Item: ForgetOwned> ForgetOwned for Leaf<Item> {
    fn forget_owned(self, index: impl Into<u64>) -> (Insert<Self>, bool) {
        let (item, forgotten) = self.0.forget_owned(index);
        (item.map(Leaf), forgotten)
    }
}

impl<Item: Height + Any> Any for Leaf<Item> {
    fn place(&self) -> Place {
        Place::Complete
    }

    fn kind(&self) -> Kind {
        Kind::Leaf
    }

    fn height(&self) -> u8 {
        <Self as Height>::Height::HEIGHT
    }

    fn children(&self) -> Vec<Insert<Child>> {
        vec![Insert::Keep(Child::new(&self.0))]
    }
}
