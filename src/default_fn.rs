use dyn_clone::DynClone;

pub trait DefaultFn<V>: DynClone {
    fn call(&self) -> V;
}

impl<F, V> DefaultFn<V> for F
where
    F: Fn() -> V + Clone,
{
    fn call(&self) -> V {
        self()
    }
}

impl<V> Default for Box<dyn DefaultFn<V>>
where
    V: Default,
{
    fn default() -> Self {
        Box::new(|| V::default())
    }
}

dyn_clone::clone_trait_object!(<V> DefaultFn<V>);
