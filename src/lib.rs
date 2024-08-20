pub extern crate bitvec;
pub extern crate j1939;
pub extern crate socketcan;

pub use bitvec::prelude::*;
pub use socketcan::{CanDataFrame, CanFrame, EmbeddedFrame, Id};

#[macro_export]
macro_rules! j1939_message {
    [$pgn:expr, $name:ident; $length:expr] => {
        pub struct $name(CanDataFrame);

        impl $name {
            pub const PGN: u32 = $pgn;
            pub const LENGTH: usize = $length;
        }

        impl TryFrom<CanDataFrame> for $name {
            type Error = ();

            fn try_from(frame: CanDataFrame) -> Result<Self, Self::Error> {
                let id = match frame.id() {
                    Id::Standard(_) => return Err(()),
                    Id::Extended(id) => j1939::Id::new(id.as_raw()),
                };

                if id.pgn_raw() == Self::PGN {
                    Ok(Self(frame))
                } else {
                    Err(())
                }
            }
        }

        impl From<$name> for CanDataFrame {
            fn from(frame: $name) -> Self {
                frame.0
            }
        }

        impl From<$name> for CanFrame {
            fn from(frame: $name) -> Self {
                CanFrame::Data(frame.0)
            }
        }

        impl AsRef<CanDataFrame> for $name {
            fn as_ref(&self) -> &CanDataFrame {
                &self.0
            }
        }
    };
}

#[macro_export]
macro_rules! j1939_messages {
    { $($name:ident[$length:expr] = $pgn:expr;)* } => {
        $(j1939_message![$pgn, $name; $length];)*
        pub enum Message {
            $($name($name),)*
        }
        impl TryFrom<CanDataFrame> for Message {
            type Error = ();
            fn try_from(frame: CanDataFrame) -> Result<Self, Self::Error> {
                let id = match frame.id() {
                    Id::Standard(_) => return Err(()),
                    Id::Extended(id) => j1939::Id::new(id.as_raw()),
                };

                let message = match id.pgn_raw() {
                    $($name::PGN => Message::$name($name(frame)),)*
                    _ => return Err(()),
                };

                Ok(message)
            }
        }
        impl From<Message> for CanDataFrame {
            fn from(message: Message) -> Self {
                match message {
                    $(Message::$name($name(frame)) => frame,)*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! can_signal {
    ($message:ident $signal:ident: $type:ty [$load:ty; $start:expr, $end:expr] ($scale:expr, $offset:expr)) => {
        impl $message {
            pub fn $signal(&self) -> $type {
                self.0.data().view_bits::<Lsb0>()[$start..$end].load_le::<$load>() as $type * $scale
                    + $offset
            }
        }
    };
}

#[macro_export]
macro_rules! inject {
    (
        $from:ty => $to:ty {
            $($fromf:ident => $tof:ident,)*
        }
    ) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                Self { $($tof: value.$fromf(),)* }
            }
        }
    }
}
