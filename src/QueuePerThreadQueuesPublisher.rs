// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Publishes to a queue used by a particular thread.
///
/// Assumes a thread-per-HyperThread model.
#[derive(Debug, Clone)]
pub struct QueuePerThreadQueuesPublisher<MessageHandlerArguments: Debug + Copy, E: Debug>
{
	queues: Arc<PerBitSetAwareData<HyperThread, Arc<Queue<MessageHandlerArguments, E>>>>,
}

unsafe impl<MessageHandlerArguments: Debug + Copy, E: Debug> Send for QueuePerThreadQueuesPublisher<MessageHandlerArguments, E>
{
}

unsafe impl<MessageHandlerArguments: Debug + Copy, E: Debug> Sync for QueuePerThreadQueuesPublisher<MessageHandlerArguments, E>
{
}

impl<MessageHandlerArguments: Debug + Copy, E: Debug> QueuePerThreadQueuesPublisher<MessageHandlerArguments, E>
{
	/// Allocate a new instance.
	#[inline(always)]
	pub fn allocate(hyper_threads: &BitSet<HyperThread>, defaults: &DefaultPageSizeAndHugePageSizes, queue_size_in_bytes: NonZeroU64, inclusive_maximum_bytes_wasted: u64) -> Self
	{
		Self
		{
			queues: Queue::queues(hyper_threads, defaults, queue_size_in_bytes, inclusive_maximum_bytes_wasted)
		}
	}

	/// Publish a message to be received by the queue for `hyper_thread_identifier`.
	///
	/// Assumes a thread-per-HyperThread model.
	///
	/// If there is no registered queue, publishes to the queue which is assumed to exist for the current thread.
	#[inline(always)]
	pub fn publish_message<MessageContents, F: FnOnce(NonNull<MessageContents>)>(&self, hyper_thread: HyperThread, compressed_type_identifier: CompressedTypeIdentifier, message_contents_constructor: F)
	{
		let queue = self.queues.get_or_current(hyper_thread);
		queue.enqueue(compressed_type_identifier, message_contents_constructor)
	}

	#[inline(always)]
	fn get_queue(&self, hyper_thread: HyperThread) -> Arc<Queue<MessageHandlerArguments, E>>
	{
		self.queues.get(hyper_thread).unwrap().clone()
	}
}
