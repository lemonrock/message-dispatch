// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


extern crate arrayvec;
extern crate cpu_affinity;
extern crate hashbrown;
extern crate libc;
extern crate magic_ring_buffer;
extern crate terminate;


use self::erased_boxed_functions::*;
use self::virtual_method_tables::*;
use ::arrayvec::ArrayVec;
use ::hashbrown::HashMap;
use ::cpu_affinity::*;
use ::magic_ring_buffer::*;
use ::std::cell::UnsafeCell;
use ::std::any::Any;
use ::std::any::TypeId;
use ::std::fmt;
use ::std::fmt::Debug;
use ::std::fmt::Formatter;
use ::std::mem::align_of;
use ::std::mem::forget;
use ::std::mem::size_of;
use ::std::mem::transmute;
use ::std::mem::uninitialized;
use ::std::ops::Deref;
use ::std::ptr::NonNull;
use ::std::ptr::null_mut;
use ::std::ptr::write;
use ::std::raw::TraitObject;
use ::std::sync::Arc;
use ::terminate::Terminate;


/// Erased, boxed functions can be used as generic message dispatchers.
pub mod erased_boxed_functions;


/// Various wrappers around virtual method tables (vtables) which allow for them to be tagged.
///
/// A tagged pointer to a vtable allows one to mix multiple `dyn Trait` (fat pointers), using the tag to differentiated the trait type.
#[allow(dead_code)]
mod virtual_method_tables;


include!("Dequeue.rs");
include!("Enqueue.rs");
include!("Message.rs");
include!("MessageHandlersRegistration.rs");
include!("MessageHeader.rs");
include!("PerThreadQueueSubscriber.rs");
include!("round_up_to_alignment.rs");
include!("Queue.rs");
include!("QueuePerThreadQueuesPublisher.rs");
include!("VariablySized.rs");
