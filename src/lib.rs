// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![deny(missing_docs)]
#![deny(unreachable_patterns)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(extern_types)]
#![feature(integer_atomics)]
#![feature(raw)]


//! #message-dispatch
//! 
//! This provides dynamic dispatch support for messages of different types and sizes sent from one thread to another (or back to the same thread) without the need to use trait objects.
//!
//! As such, the only cost involved in dispatch is the cost of an indirect call.
//!
//! It could even be used to send messages across POSIX message queues if so desired.
//!
//! Currently only implemented for Android and Linux until the underlying magic ring buffer used gains support for more Operating Systems.


use static_assertions::assert_cfg;
assert_cfg!(target_os = "linux");
assert_cfg!(target_pointer_width = "64");


use self::message::*;
use self::message_handling::*;
use self::virtual_method_tables::*;
use arrayvec::Array;
use arrayvec::ArrayVec;
use linux_support::bit_set::BitSet;
use linux_support::bit_set::PerBitSetAwareData;
use linux_support::cpu::HyperThread;
use linux_support::memory::huge_pages::DefaultPageSizeAndHugePageSizes;
use magic_ring_buffer::*;
use std::collections::HashMap;
use std::any::Any;
use std::any::TypeId;
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::mem::align_of;
use std::mem::forget;
use std::mem::size_of;
use std::mem::transmute;
#[allow(deprecated)] use std::mem::uninitialized;
use std::num::NonZeroU64;
use std::ptr::NonNull;
use std::ptr::null_mut;
use std::ptr::write;
use std::raw::TraitObject;
use std::sync::Arc;
use terminate::Terminate;


mod message;


mod message_handling;


mod virtual_method_tables;


include!("CompressedTypeIdentifier.rs");
include!("Dequeue.rs");
include!("Enqueue.rs");
include!("Message.rs");
include!("MessageHandlers.rs");
include!("Queue.rs");
include!("Queues.rs");
include!("round_up_to_alignment.rs");
include!("Subscriber.rs");
