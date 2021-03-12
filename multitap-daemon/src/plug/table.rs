use multitap_core::input;

macro_rules! Table {
    ($table:ident, $max:expr, $coded:path) => {
        pub struct $table<V>(Vec<V>) where V: Default;

        #[allow(dead_code)]
        impl<V: Default> Default for $table<V> {
            fn default() -> $table<V> {
                let count = ($max.0 as usize) + 1;
                let mut vec = Vec::with_capacity(count);
                for _ in 0..count {
                    vec.push(Default::default());
                }

                $table(vec)
            }
        }

        impl<V> std::ops::Index<u16> for $table<V>
        where
            V: Default,
        {
            type Output = V;
            fn index(&self, index: u16) -> &V {
                &self.0[index as usize]
            }
        }

        impl<V> std::ops::IndexMut<u16> for $table<V>
        where
            V: Default,
        {
            fn index_mut(&mut self, index: u16) -> &mut V {
                &mut self.0[index as usize]
            }
        }

        impl<V> std::ops::Index<$coded> for $table<V>
        where
            V: Default,
        {
            type Output = V;
            fn index(&self, index: $coded) -> &V {
                self.index(index.0)
            }
        }

        impl<V> std::ops::IndexMut<$coded> for $table<V>
        where
            V: Default,
        {
            fn index_mut(&mut self, index: $coded) -> &mut V {
                self.index_mut(index.0)
            }
        }
    };
}

Table!(Key,    input::KeyId::max(),    input::KeyId);
Table!(Axis,   input::AxisId::max(),   input::AxisId);
Table!(Motion, input::MotionId::max(), input::MotionId);
