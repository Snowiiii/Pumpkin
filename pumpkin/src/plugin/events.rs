macro_rules! event_types {
    ($($event_name:ident, $register_name:ident, $name:ident, $data:ty),+) => {
        $(
            pub trait $name: Send + Sync {
                fn $event_name(&self, data: $data);
            }

            impl<Func> $name for Func
            where
                Func: Fn($data) + Send + Sync,
            {
                fn $event_name(&self, data: $data) {
                    self(data)
                }
            }
        )*

        struct EventTypeContainer {
            $($event_name: Vec<Box<dyn $name>>,)*
        }

        impl EventRegistry {
            $(
                pub async fn $event_name(&self, data: $data) {
                    for ev in &self.container.$event_name {
                        ev.$event_name(data);
                    }
                }

                pub fn $register_name<F>(&mut self, fun: F) where F: $name + Send + Sync + 'static{
                    self.container.$event_name.push(Box::new(fun));
                }
            )*
        }

        impl Default for EventTypeContainer {
            fn default() -> Self {
                Self {
                    $(
                        $event_name: Vec::new(),
                    )*
                }
            }
        }
    };
}

event_types![on_init, register_on_init, InitEvent, ()];

#[derive(Default)]
pub struct EventRegistry {
    container: EventTypeContainer,
}
