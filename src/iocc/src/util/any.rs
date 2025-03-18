use std::any::{self, Any};
use std::ops::{Deref, DerefMut};

pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;

    fn as_any_send(&self) -> &(dyn Any + Send)
    where
        Self: Send;

    fn as_any_send_sync(&self) -> &(dyn Any + Send + Sync)
    where
        Self: Send + Sync;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_any_mut_send(&mut self) -> &mut (dyn Any + Send)
    where
        Self: Send;

    fn as_any_mut_send_sync(&mut self) -> &mut (dyn Any + Send + Sync)
    where
        Self: Send + Sync;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn into_any_send(self: Box<Self>) -> Box<dyn Any + Send>
    where
        Self: Send;

    fn into_any_send_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync>
    where
        Self: Send + Sync;

    fn type_name(&self) -> &'static str;
}

impl<T: Any> AsAny for T {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_send(&self) -> &(dyn Any + Send)
    where
        Self: Send,
    {
        self
    }

    #[inline]
    fn as_any_send_sync(&self) -> &(dyn Any + Send + Sync)
    where
        Self: Send + Sync,
    {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn as_any_mut_send(&mut self) -> &mut (dyn Any + Send)
    where
        Self: Send,
    {
        self
    }

    #[inline]
    fn as_any_mut_send_sync(&mut self) -> &mut (dyn Any + Send + Sync)
    where
        Self: Send + Sync,
    {
        self
    }

    #[inline]
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    #[inline]
    fn into_any_send(self: Box<Self>) -> Box<dyn Any + Send>
    where
        Self: Send,
    {
        self
    }

    #[inline]
    fn into_any_send_sync(self: Box<Self>) -> Box<dyn Any + Send + Sync>
    where
        Self: Send + Sync,
    {
        self
    }

    #[inline]
    fn type_name(&self) -> &'static str {
        any::type_name::<T>()
    }
}

pub trait DowncastRef {
    fn is<T: Any>(&self) -> bool;

    fn downcast_ref<T: Any>(&self) -> Option<&T>;
}

impl<S> DowncastRef for S
where
    S: Deref<Target: AsAny>,
{
    #[inline]
    fn is<T: Any>(&self) -> bool {
        (**self).as_any().is::<T>()
    }

    #[inline]
    fn downcast_ref<T: Any>(&self) -> Option<&T> {
        (**self).as_any().downcast_ref::<T>()
    }
}

pub trait DowncastMut: DowncastRef {
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T>;
}

impl<S> DowncastMut for S
where
    S: DerefMut<Target: AsAny>,
{
    #[inline]
    fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        (**self).as_any_mut().downcast_mut::<T>()
    }
}

pub trait Downcast: DowncastMut + Sized {
    type Output<T>;

    fn downcast<T: Any>(self) -> Result<Self::Output<T>, Self>;
}

impl<S> Downcast for Box<S>
where
    S: AsAny + ?Sized,
{
    type Output<T> = Box<T>;

    fn downcast<T: Any>(self) -> Result<Self::Output<T>, Self> {
        if self.is::<T>() {
            let res = self
                .into_any()
                .downcast::<T>()
                .unwrap_or_else(|_| std::unreachable!("`self` should be `Box<T>`"));
            Ok(res)
        } else {
            Err(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Trait: AsAny + Send + Sync {}

    impl Trait for i32 {}

    #[test]
    fn downcast_succeeds_when_receiver_is_a_ref() {
        let mut val = 0i32;
        let mut x: &mut dyn Trait = &mut val;

        assert_eq!(x.downcast_ref::<i32>(), Some(&0));

        *x.downcast_mut::<i32>().unwrap() = 1;
        assert_eq!(x.downcast_ref::<i32>(), Some(&1));
    }

    #[test]
    fn downcast_succeeds_when_receiver_is_a_box() {
        let mut x: Box<dyn Trait> = Box::new(0i32);

        assert_eq!(x.downcast_ref::<i32>(), Some(&0));

        *x.downcast_mut::<i32>().unwrap() = 1;
        assert_eq!(x.downcast_ref::<i32>(), Some(&1));

        let y = x.downcast::<i32>().unwrap_or(Box::new(0));
        assert_eq!(*y, 1);
    }
}
