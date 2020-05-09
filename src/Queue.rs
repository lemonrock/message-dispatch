// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A queue of variably-sized messages of different types (eg structs, traits, etc), suitable for many-writer, single consumer usage.
///
/// Ideal for a thread control queue.
#[derive(Debug)]
pub struct Queue<MessageHandlerArguments: Debug + Copy, E: Debug>
{
	magic_ring_buffer: MagicRingBuffer,
	message_handlers: UnsafeCell<MutableTypeErasedBoxedFunctionCompressedMap<MessageHandlerArguments, Result<(), E>>>,
}

impl<MessageHandlerArguments: Debug + Copy, E: Debug> Drop for Queue<MessageHandlerArguments, E>
{
	#[inline(always)]
	fn drop(&mut self)
	{
		let message_handlers = self.message_handlers();
		while
		{
			let more_data_to_read = self.magic_ring_buffer.single_reader_read_some_data::<E, _>
			(
				|buffer|
				{
					Message::process_next_message_in_buffer::<Result<(), E>, _>
					(
						buffer,
						|compressed_type_identifier, receiver|
						{
							message_handlers.drop_in_place(compressed_type_identifier, receiver);
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

impl<MessageHandlerArguments: Debug + Copy, E: Debug> Enqueue for Queue<MessageHandlerArguments, E>
{
	#[inline(always)]
	fn enqueue<MessageContents>(&self, compressed_type_identifier: CompressedTypeIdentifier, message_contents_constructor: impl FnOnce(NonNull<MessageContents>))
	{
		Message::enqueue(&self.magic_ring_buffer, compressed_type_identifier, message_contents_constructor)
	}
}

impl<MessageHandlerArguments: Debug + Copy, E: Debug> Dequeue<MessageHandlerArguments, E> for Queue<MessageHandlerArguments, E>
{
	/// Dequeues messages.
	#[inline(always)]
	fn dequeue(&self, terminate: &impl Terminate, message_handler_arguments: MessageHandlerArguments) -> Result<(), E>
	{
		let message_handlers = self.message_handlers();
		while
		{
			let more_data_to_read = self.magic_ring_buffer.single_reader_read_some_data::<E, _>
			(
				|buffer|
				{
					Message::process_next_message_in_buffer::<Result<(), E>, _>
					(
						buffer,
						|compressed_type_identifier, receiver|
						{
							message_handlers.call_and_drop_in_place(compressed_type_identifier, receiver, message_handler_arguments)
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

impl<MessageHandlerArguments: Debug + Copy, E: Debug> Queue<MessageHandlerArguments, E>
{
	/// Allocates a new `Queue`.
	#[inline(always)]
	pub fn new(defaults: &DefaultPageSizeAndHugePageSizes, buffer_size_not_page_aligned: NonZeroU64, page_size: PageSizeOrHugePageSize) -> Result<Arc<Self>, MirroredMemoryMapCreationError>
	{
		Ok
		(
			Arc::new
			(
				Self
				{
					magic_ring_buffer: MagicRingBuffer::allocate(defaults, buffer_size_not_page_aligned, page_size)?,
					message_handlers: Default::default(),
				}
			)
		)
	}

	/// New set of per-thread queues.
	#[inline(always)]
	pub fn queues(hyper_threads: &BitSet<HyperThread>, defaults: &DefaultPageSizeAndHugePageSizes, queue_size_in_bytes: NonZeroU64, page_size: PageSizeOrHugePageSize) -> Arc<PerBitSetAwareData<HyperThread, Arc<Self>>>
	{
		Arc::new
		(
			PerBitSetAwareData::new
			(
				hyper_threads,
				|_hyper_thread| Self::new(defaults, buffer_size_not_page_aligned, page_size).unwrap()
			)
		)
	}

	#[inline(always)]
	pub(crate) fn message_handlers(&self) -> &mut MutableTypeErasedBoxedFunctionCompressedMap<MessageHandlerArguments, Result<(), E>>
	{
		unsafe { &mut * self.message_handlers.get() }
	}
}
