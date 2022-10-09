#[macro_export]
macro_rules! clone_box {
    ($main_trait:ident, $clone_trait:ident) => {
        pub trait $clone_trait {
            fn clone_box(&self) -> Box<dyn $main_trait>;
        }

        impl<T> $clone_trait for T
        where
            T: 'static + $main_trait + Clone,
        {
            fn clone_box(&self) -> Box<dyn $main_trait> {
                Box::new(self.clone())
            }
        }

        impl Clone for Box<dyn $main_trait> {
            fn clone(&self) -> Box<dyn $main_trait> {
                self.clone_box()
            }
        }
    };
}
