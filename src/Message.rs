// This file is part of linux-support. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT. No part of linux-support, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2020 The developers of linux-support. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT.


/// An object that can be a message.
///
/// Messages are safely dropped if they are on a queue when the queue is dropped.
pub trait Message: Sized
{
	/// Construct message arguments.
	type ConstructMessageArguments;
	
	/// Construct a message in place using `` on a queue (used by a publishing thread).
	///
	/// This may be called on a different thread to `handle_message()`.
	unsafe fn construct_message(uninitialized_memory: NonNull<Self>, construct_message_arguments: Self::ConstructMessageArguments);
	
	/// Message handler arguments.
	type MessageHandlerArguments;
	
	/// Error that can happen when processing a dequeued message.
	type DequeuedMessageProcessingError: error::Error;
	
	/// Handle a message (used by a receiving thread).
	///
	/// This may be called on a different thread to `construct_message()`.
	///
	/// Messages are automatically dropped after this has been called.
	fn handle_message(&mut self, message_handler_arguments: &Self::MessageHandlerArguments) -> Result<(), Self::DequeuedMessageProcessingError>;
}
