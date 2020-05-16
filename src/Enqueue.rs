// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Access to the enqueue operations of a queue.
///
/// All implementations of `FixedSizeMessageBody` must share the same `DequeuedMessageProcessingError` when dequeued and processed with `message_handlers.call_and_drop_in_place()`.
trait Enqueue
{
	/// Finds a fixed size message body compressed type identifier for direct use of `enqueue()`.
	fn fixed_sized_message_body_compressed_type_identifier<FixedSizeMessageBody: 'static + Sized>(&self) -> CompressedTypeIdentifier;
	
	/// Slow but safe; unnecessary once `fixed_sized_message_body_compressed_type_identifier()` is used.
	#[inline(always)]
	fn enqueue_slow_but_safe<FixedSizeMessageBody: 'static + Sized>(&self, fixed_size_message_body_constructor: impl FnOnce(NonNull<FixedSizeMessageBody>))
	{
		let fixed_sized_message_body_compressed_type_identifier = self.fixed_sized_message_body_compressed_type_identifier::<FixedSizeMessageBody>();
		unsafe { self.enqueue(fixed_sized_message_body_compressed_type_identifier, fixed_size_message_body_constructor) }
	}
	
	/// Enqueue a message unsafely.
	unsafe fn enqueue<FixedSizeMessageBody: Sized>(&self, fixed_sized_message_body_compressed_type_identifier: CompressedTypeIdentifier, fixed_size_message_body_constructor: impl FnOnce(NonNull<FixedSizeMessageBody>));
}
