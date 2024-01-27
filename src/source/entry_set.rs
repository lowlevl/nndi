#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EntrySet<T, const N: usize> {
    slots: [Entry<T>; N],
}

impl<T, const N: usize> Default for EntrySet<T, N> {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| Default::default()),
        }
    }
}

impl<T, const N: usize> EntrySet<T, N> {
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Entry::is_vacant)
    }

    pub fn is_full(&self) -> bool {
        self.slots.iter().all(Entry::is_occupied)
    }

    fn next_vacant(&mut self) -> Option<&mut Entry<T>> {
        self.slots.iter_mut().find(|entry| entry.is_vacant())
    }

    pub fn push(&mut self, value: T) -> bool {
        self.next_vacant()
            .map(|entry| *entry = value.into())
            .is_some()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = OccupiedEntry<'_, T>> {
        self.slots
            .iter_mut()
            .filter_map(|entry| entry.is_occupied().then_some(OccupiedEntry(entry)))
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub enum Entry<T> {
    Occupied(T),

    #[default]
    Vacant,
}

impl<T> Entry<T> {
    pub fn is_vacant(&self) -> bool {
        matches!(self, Self::Vacant)
    }

    pub fn is_occupied(&self) -> bool {
        !self.is_vacant()
    }

    pub fn set(&mut self, value: T) -> Option<T> {
        let previous = self.take();

        *self = value.into();

        previous
    }

    pub fn take(&mut self) -> Option<T> {
        let mut entry = Default::default();
        std::mem::swap(self, &mut entry);

        entry.into()
    }

    pub fn clear(&mut self) {
        drop(self.take());
    }
}

impl<T> std::convert::From<T> for Entry<T> {
    fn from(value: T) -> Self {
        Entry::Occupied(value)
    }
}

impl<T> std::convert::From<Entry<T>> for Option<T> {
    fn from(value: Entry<T>) -> Self {
        match value {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant => None,
        }
    }
}

impl<'e, T> std::convert::From<&'e Entry<T>> for Option<&'e T> {
    fn from(value: &'e Entry<T>) -> Self {
        match value {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant => None,
        }
    }
}

impl<'e, T> std::convert::From<&'e mut Entry<T>> for Option<&'e mut T> {
    fn from(value: &'e mut Entry<T>) -> Self {
        match value {
            Entry::Occupied(value) => Some(value),
            Entry::Vacant => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OccupiedEntry<'e, T>(&'e mut Entry<T>);

impl<'e, T> OccupiedEntry<'e, T> {
    pub fn clear(self) {
        self.0.clear();
    }
}

impl<'e, T> std::ops::Deref for OccupiedEntry<'e, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Option::from(&*self.0).expect("The entry became vacant")
    }
}

impl<'e, T> std::ops::DerefMut for OccupiedEntry<'e, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Option::from(&mut *self.0).expect("The entry became vacant")
    }
}
