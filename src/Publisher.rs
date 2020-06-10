// This file is part of linux-support. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT. No part of linux-support, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2020 The developers of linux-support. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT.


/// A publisher for one type of message, `M`.
pub struct Publisher<'a, M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	map: PerBitSetAwareData<HyperThread, (&'a Queue<MessageHandlerArguments, DequeuedMessageProcessingError>, CompressedTypeIdentifier)>,
	default_hyper_thread: HyperThread,
	marker: PhantomData<M>,
}

impl<'a, M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Publisher<'a, M, MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn new(queue_data: &'a PerBitSetAwareData<HyperThread, Queue<MessageHandlerArguments, DequeuedMessageProcessingError>>, default_hyper_thread: HyperThread) -> Self
	{
		Self
		{
			map: queue_data.map_ref(|_hyper_thread, queue| (queue, queue.fixed_sized_message_body_compressed_type_identifier::<M>())),
			default_hyper_thread,
			marker: PhantomData,
		}
	}
	
	/// A publisher publishes to a specific hyper thread.
	///
	/// If there is no queue for the hyper thread, publishes to itself.
	/// This supports a scenario under Linux using the `SO_INCOMING_CPU` socket option, which can map to a CPU not assigned to the process.
	///
	/// Returns the actual hyper thread published to.
	#[inline(always)]
	pub fn publish(&self, hyper_thread: HyperThread, construct_message_arguments: M::ConstructMessageArguments)-> HyperThread
	{
		let (&(queue, fixed_sized_message_body_compressed_type_identifier), actual_hyper_thread) = self.map.get_or(hyper_thread, self.default_hyper_thread);
		
		unsafe { queue.enqueue(fixed_sized_message_body_compressed_type_identifier, |uninitialized_memory| M::construct_message(uninitialized_memory, construct_message_arguments)) };
		actual_hyper_thread
	}
}
