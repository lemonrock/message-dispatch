// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A queue of variably-sized messages of different types (eg structs, traits, etc), suitable for many-writer, single consumer usage.
///
/// Ideal for a thread control queue.
///
/// `MessageHandlerArguments` must be common to all possible message types (all possible `FixedSizeMessageBody` and `CompressedTypeIdentifier`s).
/// `DequeuedMessageProcessingError` must be common to all possible message types (all possible `FixedSizeMessageBody` and `CompressedTypeIdentifier`s).
///
/// Both a sending thread and the receiving thread have to agree on `message_handlers` so that `drop()` can work.
#[derive(Debug)]
struct Queue<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	magic_ring_buffer: MagicRingBuffer,
	message_handlers: MessageHandlers<MessageHandlerArguments, Result<(), DequeuedMessageProcessingError>>,
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Drop for Queue<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let message_handlers = self.message_handlers();
		while
		{
			let more_data_to_read = self.magic_ring_buffer.single_reader_read_some_data::<DequeuedMessageProcessingError, _>
			(
				|buffer|
				{
					MessageRepresentation::process_next_message_in_buffer::<Result<(), DequeuedMessageProcessingError>, _>
					(
						buffer,
						|compressed_type_identifier, variably_sized_message_body|
						{
							message_handlers.drop_in_place(compressed_type_identifier, variably_sized_message_body);
							Ok(())
						}
					)
				}
			).expect("Should never happen");

			more_data_to_read
		}
		{
		}
	}
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Enqueue for Queue<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn fixed_sized_message_body_compressed_type_identifier<FixedSizeMessageBody: 'static + Sized>(&self) -> CompressedTypeIdentifier
	{
		self.message_handlers.find_fixed_size_message_body_compressed_type_identifier::<FixedSizeMessageBody>().expect("Unregistered FixedSizeMessageBody")
	}
	
	#[inline(always)]
	unsafe fn enqueue<FixedSizeMessageBody: Sized>(&self, fixed_sized_message_body_compressed_type_identifier: CompressedTypeIdentifier, fixed_size_message_body_constructor: impl FnOnce(NonNull<FixedSizeMessageBody>))
	{
		MessageRepresentation::enqueue(&self.magic_ring_buffer, fixed_sized_message_body_compressed_type_identifier, fixed_size_message_body_constructor)
	}
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Dequeue<MessageHandlerArguments, DequeuedMessageProcessingError> for Queue<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	/// Dequeues messages.
	#[inline(always)]
	fn dequeue(&self, terminate: &Arc<impl Terminate>, message_handler_arguments: &MessageHandlerArguments) -> Result<(), DequeuedMessageProcessingError>
	{
		let message_handlers = self.message_handlers();
		while
		{
			let more_data_to_read = self.magic_ring_buffer.single_reader_read_some_data::<DequeuedMessageProcessingError, _>
			(
				|buffer|
				{
					MessageRepresentation::process_next_message_in_buffer::<Result<(), DequeuedMessageProcessingError>, _>
					(
						buffer,
						|compressed_type_identifier, variably_sized_message_body|
						{
							message_handlers.call_and_drop_in_place(compressed_type_identifier, variably_sized_message_body, message_handler_arguments.clone())
						}
					)
				}
			)?;

			more_data_to_read && terminate.should_continue()
		}
		{
		}

		Ok(())
	}
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Queue<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	/// Allocates a new `Queue`.
	#[inline(always)]
	pub(crate) fn new(message_handlers: MessageHandlers<MessageHandlerArguments, Result<(), DequeuedMessageProcessingError>>, defaults: &DefaultPageSizeAndHugePageSizes, queue_size_in_bytes: NonZeroU64, inclusive_maximum_bytes_wasted: u64) -> Result<Self, MirroredMemoryMapCreationError>
	{
		Ok
		(
			Self
			{
				magic_ring_buffer: MagicRingBuffer::allocate(defaults, queue_size_in_bytes, inclusive_maximum_bytes_wasted)?,
				message_handlers,
			}
		)
	}
	
	#[inline(always)]
	fn message_handlers(&self) -> &MessageHandlers<MessageHandlerArguments, Result<(), DequeuedMessageProcessingError>>
	{
		&self.message_handlers
	}
}
