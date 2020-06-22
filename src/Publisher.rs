// This file is part of linux-support. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT. No part of linux-support, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2020 The developers of linux-support. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT.


/// A publisher for one type of message, `M`.
///
/// `queues_mapped` holds a reference to `_queues_drop_reference`, making this a self-referential struct.
/// As `_queues_drop_reference` is internally an `Arc`, this is ok as the reference is to a stable location in memory, ie one that doesn't move.
/// This can not be expressed using lifetimes, hence the `*const Queue` below (otherwise the lifetime would be `'self', if such a thing existed).
#[derive(Debug)]
pub struct Publisher<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	_queues_drop_reference: Queues<MessageHandlerArguments, DequeuedMessageProcessingError>,
	queues_mapped: PerBitSetAwareData<HyperThread, (*const Queue<MessageHandlerArguments, DequeuedMessageProcessingError>, CompressedTypeIdentifier)>,
	default_hyper_thread: HyperThread,
	marker: PhantomData<M>,
}

impl<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Publisher<M, MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn new(queues: &Queues<MessageHandlerArguments, DequeuedMessageProcessingError>, default_hyper_thread: HyperThread) -> Self
	{
		Self
		{
			_queues_drop_reference: queues.clone(),
			queues_mapped: queues.0.map_ref(|_hyper_thread, queue| (queue as *const _, queue.fixed_sized_message_body_compressed_type_identifier::<M>())),
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
		let (&(queue, fixed_sized_message_body_compressed_type_identifier), actual_hyper_thread) = self.queues_mapped.get_or(hyper_thread, self.default_hyper_thread);
		
		unsafe { (& * queue).enqueue(fixed_sized_message_body_compressed_type_identifier, |uninitialized_memory| M::construct_message(uninitialized_memory, construct_message_arguments)) };
		actual_hyper_thread
	}
}
