//! A concurrent type map for used for caching and protocol extensions.

// Based on `Extensions` from the `http` create, released under the Apache v2 and MIT licenses
//
// Source:
// https://github.com/hyperium/http/blob/63102bcd29fcd4a094cac6a4afb0af370ef1fcbe/src/extensions.rs

use std::{
    any::{
        Any,
        TypeId,
    },
    fmt,
    hash::{
        BuildHasherDefault,
        Hasher,
    },
};

use dashmap::{
    DashMap,
    mapref::one::{
        MappedRef,
        MappedRefMut,
    },
};

type Key = TypeId;
type Value = Box<dyn AnyClone + Send + Sync>;
type AnyMap = DashMap<Key, Value, BuildHasherDefault<IdHasher>>;

// With TypeIds as keys, there's no need to hash them. They are already hashes themselves, coming
// from the compiler. The IdHasher just holds the u64 of the TypeId, and then returns it, instead
// of doing any bit fiddling.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// A concurrent type map used for caching and protocol extensions.
#[derive(Clone, Default)]
pub struct ConcurrentExtensions {
    map: Box<AnyMap>,
}

impl ConcurrentExtensions {
    /// Creates an empty `ConcurrentExtensions`.
    #[inline]
    pub fn new() -> ConcurrentExtensions {
        ConcurrentExtensions {
            map: Box::default(),
        }
    }

    /// Inserts a type into the map.
    ///
    /// If a extension of this type already existed, it will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// assert!(ext.insert(5i32).is_none());
    /// assert!(ext.insert(4u8).is_none());
    /// assert_eq!(ext.insert(9i32), Some(5i32));
    /// ```
    pub fn insert<T: Clone + Send + Sync + 'static>(&self, val: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| boxed.into_any().downcast().ok().map(|boxed| *boxed))
    }

    /// Gets a reference to a type previously inserted in the map.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// assert!(ext.get::<i32>().is_none());
    /// ext.insert(5i32);
    ///
    /// assert_eq!(ext.get::<i32>().as_deref(), Some(&5i32));
    /// ```
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<MappedRef<'_, Key, Value, T>> {
        self.map.get(&TypeId::of::<T>()).map(|dm_ref| {
            dm_ref.map(|boxed| (**boxed).as_any().downcast_ref().expect("type should be T"))
        })
    }

    /// Gets a mutable reference to a type previously inserted in the map.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// ext.insert(String::from("Hello"));
    /// ext.get_mut::<String>().unwrap().push_str(" World");
    ///
    /// assert_eq!(ext.get::<String>().as_deref().unwrap(), "Hello World");
    /// ```
    pub fn get_mut<T: Send + Sync + 'static>(&self) -> Option<MappedRefMut<'_, Key, Value, T>> {
        self.map.get_mut(&TypeId::of::<T>()).map(|dm_ref_mut| {
            dm_ref_mut.map(|boxed| {
                (**boxed)
                    .as_any_mut()
                    .downcast_mut()
                    .expect("value type should be T")
            })
        })
    }

    /// Gets a mutable reference to a type, inserting `value` if not already present.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// *ext.get_or_insert(1i32) += 2;
    ///
    /// assert_eq!(*ext.get::<i32>().unwrap(), 3);
    /// ```
    pub fn get_or_insert<T: Clone + Send + Sync + 'static>(
        &self,
        value: T,
    ) -> MappedRefMut<'_, Key, Value, T> {
        self.get_or_insert_with(|| value)
    }

    /// Gets a mutable reference to a type, inserting the value created by `f` if not already
    /// present.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// *ext.get_or_insert_with(|| 1i32) += 2;
    ///
    /// assert_eq!(*ext.get::<i32>().unwrap(), 3);
    /// ```
    pub fn get_or_insert_with<T: Clone + Send + Sync + 'static, F: FnOnce() -> T>(
        &self,
        f: F,
    ) -> MappedRefMut<'_, Key, Value, T> {
        self.map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(f()))
            .map(|boxed| {
                (**boxed)
                    .as_any_mut()
                    .downcast_mut()
                    .expect("value type should be T")
            })
    }

    /// Gets a mutable reference to a type, inserting the type's default value if not already
    /// present.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// *ext.get_or_insert_default::<i32>() += 2;
    ///
    /// assert_eq!(*ext.get::<i32>().unwrap(), 2);
    /// ```
    pub fn get_or_insert_default<T: Default + Clone + Send + Sync + 'static>(
        &self,
    ) -> MappedRefMut<'_, Key, Value, T> {
        self.get_or_insert_with(T::default)
    }

    /// Removes a type from the map.
    ///
    /// If a extension of this type existed, it will be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// ext.insert(5i32);
    /// assert_eq!(ext.remove::<i32>(), Some(5i32));
    /// assert!(ext.get::<i32>().is_none());
    /// ```
    pub fn remove<T: Send + Sync + 'static>(&self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.1.into_any().downcast().ok().map(|boxed| *boxed))
    }

    /// Clears the map of all inserted extensions.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// ext.insert(5i32);
    /// ext.clear();
    ///
    /// assert!(ext.get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn clear(&self) {
        self.map.clear();
    }

    /// Checks whether the map is empty or not.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// assert!(ext.is_empty());
    /// ext.insert(5i32);
    /// assert!(!ext.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets the numer of types available in the map.
    ///
    /// # Example
    ///
    /// ```
    /// # use concurrent_extensions::ConcurrentExtensions;
    /// let mut ext = ConcurrentExtensions::new();
    /// assert_eq!(ext.len(), 0);
    /// ext.insert(5i32);
    /// assert_eq!(ext.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl fmt::Debug for ConcurrentExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentExtensions").finish()
    }
}

pub trait AnyClone: Any {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Clone + Send + Sync + 'static> AnyClone for T {
    fn clone_box(&self) -> Box<dyn AnyClone + Send + Sync> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl Clone for Box<dyn AnyClone + Send + Sync> {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::DerefMut;

    use super::*;

    #[test]
    fn concurrent_extensions() {
        #[derive(Clone, Debug, PartialEq)]
        struct MyType(i32);

        #[derive(Clone, Debug, PartialEq)]
        struct MyMemoizedType(i32);

        #[derive(Clone, Debug, PartialEq)]
        struct MyMemoizedTypeWith(i32);

        #[derive(Clone, Debug, PartialEq)]
        struct MyMemoizedTypeDefault(i32);

        impl Default for MyMemoizedTypeDefault {
            fn default() -> Self {
                Self(7)
            }
        }

        let extensions = ConcurrentExtensions::new();

        extensions.insert(5i32);
        extensions.insert(MyType(10));

        assert_eq!(extensions.get().as_deref(), Some(&5i32));
        assert_eq!(extensions.get_mut().as_deref_mut(), Some(&mut 5i32));

        assert_eq!(extensions.get::<MyMemoizedType>().as_deref(), None);
        assert_eq!(
            extensions.get_or_insert(MyMemoizedType(10)).deref_mut(),
            &mut MyMemoizedType(10)
        );
        assert_eq!(extensions.get().as_deref(), Some(&MyMemoizedType(10)));

        assert_eq!(extensions.get::<MyMemoizedTypeWith>().as_deref(), None);
        assert_eq!(
            extensions
                .get_or_insert_with(|| MyMemoizedTypeWith(12))
                .deref_mut(),
            &mut MyMemoizedTypeWith(12)
        );
        assert_eq!(extensions.get().as_deref(), Some(&MyMemoizedTypeWith(12)));

        assert_eq!(extensions.get::<MyMemoizedTypeDefault>().as_deref(), None);
        assert_eq!(
            extensions
                .get_or_insert_default::<MyMemoizedTypeDefault>()
                .deref_mut(),
            &mut MyMemoizedTypeDefault(7)
        );
        assert_eq!(extensions.get().as_deref(), Some(&MyMemoizedTypeDefault(7)));

        let ext2 = extensions.clone();

        assert_eq!(extensions.remove::<i32>(), Some(5i32));
        assert!(extensions.get::<i32>().is_none());

        // clone still has it
        assert_eq!(ext2.get().as_deref(), Some(&5i32));
        assert_eq!(ext2.get().as_deref(), Some(&MyType(10)));

        assert_eq!(extensions.get::<bool>().as_deref(), None);
        assert_eq!(extensions.get().as_deref(), Some(&MyType(10)));

        assert!(!extensions.is_empty());

        extensions.clear();

        assert!(extensions.is_empty());

        assert_eq!(extensions.get::<MyType>().as_deref(), None);
        // clone still has it
        assert_eq!(ext2.get().as_deref(), Some(&MyType(10)));
    }
}
