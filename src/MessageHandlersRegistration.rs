// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Used to pass a constructor across threads.
pub trait MessageHandlersRegistration
{
	/// Message handler arguments type.
	type MessageHandlerArguments: Debug + Copy;

	/// Error type.
	type E: Debug;

	/// Arguments passed when registering.
	type Arguments;

	/// Register all messages handlers.
	fn register_all_message_handlers(&self, register: &mut impl Register<Self::MessageHandlerArguments, Result<(), Self::E>>, arguments: &Self::Arguments);
}
