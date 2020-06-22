// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


pub(crate) struct MessageHandler<MessageHandlerArguments, MessageHandlerReturns>
{
	message_handler: fn(NonNull<VariablySizedMessageBody>, &MessageHandlerArguments) -> MessageHandlerReturns,
}

impl<MessageHandlerArguments, MessageHandlerReturns> Debug for MessageHandler<MessageHandlerArguments, MessageHandlerReturns>
{
	#[inline(always)]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result
	{
		write!(f, "MessageHandler({:?})", self.message_handler as usize)
	}
}

impl<MessageHandlerArguments, MessageHandlerReturns> MessageHandler<MessageHandlerArguments, MessageHandlerReturns>
{
	/// Creates a new instance, wrapping `function`.
	///
	/// `function` will be moved from the stack to the heap.
	#[inline(always)]
	pub(crate) fn new<FixedSizedMessageBody: Sized>(message_handler: fn(&mut FixedSizedMessageBody, &MessageHandlerArguments) -> MessageHandlerReturns) -> Self
	{
		Self
		{
			message_handler: unsafe { transmute(message_handler) },
		}
	}

	/// A very dangerous method that will fail in subtle yet fatal ways if `VariablySizedMessageBody` is not the same type as `FixedSizedMessageBody` in `new()`.
	#[inline(always)]
	pub(crate) fn call(&self, variably_sized_message_body: NonNull<VariablySizedMessageBody>, arguments: &MessageHandlerArguments) -> MessageHandlerReturns
	{
		(self.message_handler)(variably_sized_message_body, arguments)
	}
}
