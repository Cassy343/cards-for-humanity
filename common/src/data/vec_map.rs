use std::{
    borrow::Borrow,
    ops::{Deref, DerefMut, Index, IndexMut},
    slice::SliceIndex,
};

pub struct VecMap<K, V> {
    inner: Vec<(K, V)>,
}

impl<K: Eq, V> VecMap<K, V> {
    pub fn new() -> Self {
        VecMap { inner: Vec::new() }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.inner.iter().find(|(k, _)| k.borrow() == key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.inner.iter().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.inner.iter().map(|(_, value)| value)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.inner.iter_mut().map(|(_, value)| value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.inner.iter_mut().find(|(k, _)| k == &key) {
            Some((_, val)) => *val = value,
            None => self.inner.push((key, value)),
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.inner
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, val)| val)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.inner
            .iter_mut()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, val)| val)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        Some(
            self.inner
                .remove(self.inner.iter().position(|(k, _)| k.borrow() == key)?)
                .1,
        )
    }
}

impl<K, V> Deref for VecMap<K, V> {
    type Target = [(K, V)];

    fn deref(&self) -> &Self::Target {
        Deref::deref(&self.inner)
    }
}

impl<K, V> DerefMut for VecMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        DerefMut::deref_mut(&mut self.inner)
    }
}

impl<K, V, I: SliceIndex<[(K, V)]>> Index<I> for VecMap<K, V> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.inner, index)
    }
}

impl<K, V, I: SliceIndex<[(K, V)]>> IndexMut<I> for VecMap<K, V> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.inner, index)
    }
}
