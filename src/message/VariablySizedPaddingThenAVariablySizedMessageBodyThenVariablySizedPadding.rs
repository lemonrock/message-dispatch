// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


extern
{
	/// `VariablySizedPadding`, then a `VariablySizedMessageBody`, then `VariablySizedPadding`.
	///
	/// Impossible to represent three variably sized types in a struct in Rust, hence this is defined as an extern type rather than:-
	///
	/// ```
	/// struct VariablySizedPaddingThenAVariablySizedMessageBodyThenVariablySizedPadding
	/// {
	/// 	variably_sized_padding_before: VariablySizedPadding,
	/// 	variably_sized_message_body: VariablySizedMessageBody,
	/// 	variably_sized_padding_after: VariablySizedPadding,
	/// }
	/// ```
	type VariablySizedPaddingThenAVariablySizedMessageBodyThenVariablySizedPadding;
}

impl Debug for VariablySizedPaddingThenAVariablySizedMessageBodyThenVariablySizedPadding
{
	#[inline(always)]
	fn fmt(&self, f: &mut Formatter) -> fmt::Result
	{
		write!(f, "VariablySizedPaddingThenAVariablySizedMessageBodyThenVariablySizedPadding {{ variably_sized_padding_before: _, variably_sized_message_body: _, variably_sized_padding_after: _ }}")
	}
}
