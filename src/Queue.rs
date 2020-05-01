// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


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
	pub fn allocate_from_dev_shm(file_extension: &str, queue_size_in_bytes: usize) -> Result<Arc<Self>, MirroredMemoryMapCreationError>
	{
		Ok
		(
			Arc::new
			(
				Self
				{
					magic_ring_buffer: MagicRingBuffer::allocate_mirrored_and_not_swappable_from_dev_shm(file_extension, queue_size_in_bytes)?,
					message_handlers: Default::default(),
				}
			)
		)
	}

	/// New set of per-thread queues.
	#[inline(always)]
	pub fn queues(hyper_threads: &BitSet<HyperThread>, queue_size_in_bytes: usize) -> Arc<PerBitSetAwareData<HyperThread, Arc<Self>>>
	{
		Arc::new
		(
			PerBitSetAwareData::new
			(
				hyper_threads,
				|_hyper_thread| Self::allocate_from_dev_shm("queue", queue_size_in_bytes).unwrap()
			)
		)
	}

	#[inline(always)]
	pub(crate) fn message_handlers(&self) -> &mut MutableTypeErasedBoxedFunctionCompressedMap<MessageHandlerArguments, Result<(), E>>
	{
		unsafe { &mut * self.message_handlers.get() }
	}
}
