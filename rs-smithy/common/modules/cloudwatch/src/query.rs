use aws_sdk_cloudwatchlogs;

/*
 CloudWatch Logs Insights supports different log types. For every log that's sent to Amazon CloudWatch Logs, CloudWatch Logs Insights automatically generates five system fields:

    @message contains the raw unparsed log event. This is the equivalent to the message field in InputLogevent.

    @timestamp contains the event timestamp in the log event's timestamp field. This is the equivalent to the timestamp field in InputLogevent.

    @ingestionTime contains the time when CloudWatch Logs received the log event.

    @logStream contains the name of the log stream that the log event was added to. Log streams group logs through the same process that generated them.

    @log is a log group identifier in the form of account-id:log-group-name. When querying multiple log groups, this can be usefule to identify which log group a particular event belongs to.
 */

 // Nested fields in logs are located using dot notation
 //




// Query syntax notes:
// You can include one or more query commands separated by Unix-style pipe characters (|) in your queries.


// TODO: the `intrinsics` file needs some work