// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A map that maps Types (not instances) to function closures that handle instances of those Types.
///
/// Holds state that lives longer than each call to a function closure.
///
/// Can not hold more than 256 functions, but this restriction makes it perform quicker.
///
/// What do the various type arguments relate to?
///
/// * `MessageHandlerArguments`: This is short-lived data that is passed by move every time to a call to a function closure, `Function`.
/// * `MessageHandlerReturns` is the result type of calling a function closure, `Function`. Typically it will be `Result<X, Y>`.
/// * `MessageHandler` is the type of a function closure that takes an instance, `FixedSizedMessageBody` ('&mut Self') and arguments `MessageHandlerArguments`.
/// * `FixedSizedMessageBody` is the instance of a Type.
///
/// `MessageHandlerArguments` and `MessageHandlerReturns` have to be the same for all registered function closures.
/// `MessageHandler` and `FixedSizedMessageBody` are of a different type for each registered function closure.
///
/// A very clever optimization of this structure could produce a jump table at runtime, so reducing indirect calls to direct calls, should this be necessary.
#[derive(Debug)]
pub struct MessageHandlers<MessageHandlerArguments, MessageHandlerReturns>
{
	compressed_type_identifier_to_function: ArrayVec<[(MessageHandler<MessageHandlerArguments, MessageHandlerReturns>, DropVariablySizedMessageBodyInPlaceFunctionPointer); CompressedTypeIdentifier::ExclusiveMaximum]>,
	type_identifier_to_compressed_type_identifier: HashMap<TypeId, CompressedTypeIdentifier>,
	largest_possible_message: NonZeroU64,
}

impl<MessageHandlerArguments, MessageHandlerReturns> Default for MessageHandlers<MessageHandlerArguments, MessageHandlerReturns>
{
	#[inline(always)]
	fn default() -> Self
	{
		Self
		{
			compressed_type_identifier_to_function: ArrayVec::default(),
			type_identifier_to_compressed_type_identifier: HashMap::with_capacity(CompressedTypeIdentifier::ExclusiveMaximum),
			largest_possible_message: unsafe { NonZeroU64::new_unchecked(MessageRepresentation::smallest_possible_total_message_size_including_message_header() as u64) },
		}
	}
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> MessageHandlers<MessageHandlerArguments, Result<(), DequeuedMessageProcessingError>>
{
	/// Registers a `MessageHandler` and returns a `CompressedTypeIdentifier` to refer to it.
	///
	/// All registered `MessageHandler` receive the same type of `Arguments` but receive specific types of `FixedSizedMessageBody`.
	///
	/// `CompressedTypeIdentifier` are monotonically increasing from 0 (inclusive), so they can be predicted.
	/// There is a 1:1 relationship between `Message` and `CompressedTypeIdentifier`; they are linked through the `Message`'s `TypeId`.
	///
	/// If `debug_assertions` are configured, panics if the `MessageHandler` has already been registered.
	/// If `debug_assertions` are configured, panics if there is not space for more `MessageHandler`s (only 256 message handlers are allowed).
	///
	/// A `MessageHandler` does not need to call `drop_in_place()` on `Message`; this will be done when the `MessageHandler` returns.
	#[inline(always)]
	pub fn register_message_handler<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>>(&mut self) -> CompressedTypeIdentifier
	{
		self.register_message_handler_internal::<M>(M::handle_message)
	}
}

impl<MessageHandlerArguments, MessageHandlerReturns> MessageHandlers<MessageHandlerArguments, MessageHandlerReturns>
{
	#[inline(always)]
	fn register_message_handler_internal<FixedSizedMessageBody: 'static + Sized>(&mut self, message_handler: fn(&mut FixedSizedMessageBody, &MessageHandlerArguments) -> MessageHandlerReturns) -> CompressedTypeIdentifier
	{
		let next_compressed_type_identifier = CompressedTypeIdentifier::next(&self.compressed_type_identifier_to_function);
		
		{
			let type_identifier = TypeId::of::<FixedSizedMessageBody>();
			let previous = self.type_identifier_to_compressed_type_identifier.insert(type_identifier, next_compressed_type_identifier);
			debug_assert!(previous.is_none(), "Duplicate registration");
		};
		
		let drop_in_place_function_pointer =
		{
			let virtual_method_table_pointer = VirtualMethodTablePointer::from_any::<FixedSizedMessageBody>();
			unsafe { transmute(virtual_method_table_pointer.drop_in_place_function_pointer()) }
		};
		
		self.compressed_type_identifier_to_function.push((MessageHandler::new(message_handler), drop_in_place_function_pointer));
		
		{
			let largest_possible_message = unsafe { NonZeroU64::new_unchecked(MessageRepresentation::largest_possible_total_message_size_including_message_header::<FixedSizedMessageBody>() as u64) };
			if largest_possible_message > self.largest_possible_message
			{
				self.largest_possible_message = largest_possible_message
			}
		}
		
		next_compressed_type_identifier
	}
	
	/// Finds a compressed type identifier for a given type.
	///
	/// Slow as it uses a HashMap look up; do not do this on the critical path.
	#[inline(always)]
	pub fn find_fixed_size_message_body_compressed_type_identifier<FixedSizeMessageBody: 'static + Sized>(&self) -> Option<CompressedTypeIdentifier>
	{
		let type_identifier = TypeId::of::<FixedSizeMessageBody>();
		self.find_fixed_size_message_body_compressed_type_identifier_from_type_identifier(type_identifier)
	}
	
	#[inline(always)]
	pub(crate) fn queue_size_in_bytes(&self, preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body: NonZeroU64) -> NonZeroU64
	{
		unsafe { NonZeroU64::new_unchecked(self.largest_possible_message.get() * preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body.get()) }
	}
	
	/// Calls the function registered for this compressed type identifier.
	///
	/// `variably_sized_message_body` has a known size if `compressed_type_identifier` is known.
	///
	/// Panics if no function is registered (only if `debug_assertions` are configured).
	#[inline(always)]
	pub(crate) fn call_and_drop_in_place(&self, compressed_type_identifier: CompressedTypeIdentifier, variably_sized_message_body: NonNull<VariablySizedMessageBody>, message_handler_arguments: &MessageHandlerArguments) -> MessageHandlerReturns
	{
		let (message_handler, drop_in_place_function_pointer) = self.entry(compressed_type_identifier);
		let result = message_handler.call(variably_sized_message_body, message_handler_arguments);
		Self::drop_message(drop_in_place_function_pointer, variably_sized_message_body);
		result
	}

	/// Calls the drop in place function registered for this compressed type identifier.
	///
	/// `variably_sized_message_body` has a known size if `compressed_type_identifier` is known.
	///
	/// Panics if no function is registered (only if `debug_assertions` are configured).
	#[inline(always)]
	pub(crate) fn drop_in_place(&self, compressed_type_identifier: CompressedTypeIdentifier, variably_sized_message_body: NonNull<VariablySizedMessageBody>)
	{
		let (_message_handler, drop_in_place_function_pointer) = self.entry(compressed_type_identifier);
		Self::drop_message(drop_in_place_function_pointer, variably_sized_message_body)
	}
	
	#[inline(always)]
	fn drop_message(drop_in_place_function_pointer: &DropVariablySizedMessageBodyInPlaceFunctionPointer, variably_sized_message_body: NonNull<VariablySizedMessageBody>)
	{
		drop_in_place_function_pointer(variably_sized_message_body)
	}

	/// Finds the function registered for this compressed type identifier.
	///
	/// Panics if no function is registered (only if `debug_assertions` are configured).
	#[inline(always)]
	fn entry(&self, compressed_type_identifier: CompressedTypeIdentifier) -> &(MessageHandler<MessageHandlerArguments, MessageHandlerReturns>, DropVariablySizedMessageBodyInPlaceFunctionPointer)
	{
		let index = compressed_type_identifier.index();

		if cfg!(debug_assertions)
		{
			self.compressed_type_identifier_to_function.get(index).unwrap()
		}
		else
		{
			unsafe { self.compressed_type_identifier_to_function.get_unchecked(index) }
		}
	}
	
	#[inline(always)]
	fn find_fixed_size_message_body_compressed_type_identifier_from_type_identifier(&self, type_identifier: TypeId) -> Option<CompressedTypeIdentifier>
	{
		self.type_identifier_to_compressed_type_identifier.get(&type_identifier).map(|value| *value)
	}
}
