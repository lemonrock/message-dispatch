// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A wrapper to hold a `FnMut(Receiver) -> R` closure which erases the type of `Receiver` so that multiple instances can be created and used as, say, handlers of different messages in maps.
pub(crate) struct MutableTypeErasedBoxedFunction<MessageHandlerArguments, MessageHandlerReturns>
{
	boxed_function_pointer: NonNull<BoxedFunctionPointer>,
	call_boxed_function_pointer: fn(NonNull<BoxedFunctionPointer>, NonNull<VariablySizedMessageBody>, &MessageHandlerArguments) -> MessageHandlerReturns,
	drop_boxed_function_pointer: fn(NonNull<BoxedFunctionPointer>),
}

impl<MessageHandlerArguments, MessageHandlerReturns> Drop for MutableTypeErasedBoxedFunction<MessageHandlerArguments, MessageHandlerReturns>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		(self.drop_boxed_function_pointer)(self.boxed_function_pointer)
	}
}

impl<MessageHandlerArguments, MessageHandlerReturns> MutableTypeErasedBoxedFunction<MessageHandlerArguments, MessageHandlerReturns>
{
	/// Creates a new instance, wrapping `function`.
	///
	/// `function` will be moved from the stack to the heap.
	#[inline(always)]
	pub(crate) fn new<MessageHandler: FnMut(&mut FixedSizedMessageBody, &MessageHandlerArguments) -> MessageHandlerReturns, FixedSizedMessageBody: Sized>(message_handler: MessageHandler) -> Self
	{
		let call_boxed_function_pointer: for<'message_handler, 'message_body, 'message_arguments> fn(&'message_handler mut MessageHandler, &'message_body mut FixedSizedMessageBody, &'message_arguments MessageHandlerArguments) -> MessageHandlerReturns = Self::call_boxed_function::<MessageHandler, FixedSizedMessageBody>;
		let drop_boxed_function_pointer: fn(NonNull<MessageHandler>) = Self::drop_boxed_function::<MessageHandler, FixedSizedMessageBody>;
		
		unsafe
		{
			Self
			{
				boxed_function_pointer: NonNull::new_unchecked(Box::into_raw(Box::new(message_handler)) as *mut BoxedFunctionPointer),
				call_boxed_function_pointer: transmute(call_boxed_function_pointer),
				drop_boxed_function_pointer: transmute(drop_boxed_function_pointer),
			}
		}
	}

	/// A very dangerous method that will fail in subtle yet fatal ways if `variably_sized_message_body` is not the same type used when `new()` was called.
	///
	/// As the whole purpose of this struct is to erase the type of `variably_sized_message_body`, this requirement is not enforced by the type system.
	#[inline(always)]
	pub(crate) fn call(&self, variably_sized_message_body: NonNull<VariablySizedMessageBody>, arguments: &MessageHandlerArguments) -> MessageHandlerReturns
	{
		(self.call_boxed_function_pointer)(self.boxed_function_pointer, variably_sized_message_body, arguments)
	}
	
	#[inline(always)]
	fn call_boxed_function<MessageHandler: FnMut(&mut FixedSizedMessageBody, &MessageHandlerArguments) -> MessageHandlerReturns, FixedSizedMessageBody: Sized>(message_handler: &mut MessageHandler, receiver: &mut FixedSizedMessageBody, arguments: &MessageHandlerArguments) -> MessageHandlerReturns
	{
		message_handler(receiver, arguments)
	}
	
	#[inline(always)]
	fn drop_boxed_function<MessageHandler: FnMut(&mut FixedSizedMessageBody, &MessageHandlerArguments) -> MessageHandlerReturns, FixedSizedMessageBody: Sized>(boxed_function_pointer: NonNull<MessageHandler>)
	{
		drop(unsafe { Box::from_raw(boxed_function_pointer.as_ptr()) });
	}
	
}
