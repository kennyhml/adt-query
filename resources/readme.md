Useful resources found poking around in the ADT Plugin Source, analyzing network traffic and testing endpoints.


# Statefulness

## What is it used for
Stateful sessions are required to perform operations on the sap system that rely on context across multiple requests. 

The most common example for this is modifying objects. To modify an object, you must first lock it for the duration of your modifications.

This is done through the `<object>?_action=LOCK&accessMode=MODIFY` endpoint which then provides you with a `lockHandle`. 
If you do this without setting the `stateful` header and observe your locks in SM12, you will notice that no persistent lock is created. 
If you attempt to update the object using the handle, you receive an error that the lock is not valid.

The reason for this should be apparent. Locks serve the purpose to prevent multiple users from editing an object at once. They should only exist for the duration
of a session that a user is actually editing the object. If the user forgets to manually unlock the object, or the client fails to do so, the object would remain locked.

Thus the locks are bound to a session, which in itself is also bound to a timeout. When the session expires, so do the associated locks. This ensures objects remain locked unintentionally.

## Technical Details
Whether the session is stateful or statless is determined through the `X-sap-adt-sessiontype` header: `stateful` or `stateless`.

Transaction SM04 allows you to view the sessions, it also shows that Eclipse still uses RFC Sessions
Transaction SM12 allows to see what locks a user has in place

`sap-contextid` is the cookie that is set upon creating a stateful session and is responsible for the context of that session.
`sap/bc/adt/core/http/sessions` shows session handling information, NOT ACTIVE SESSIONS! 
You can also log off at `/sap/public/bc/icf/logoff` which invalidates your session and releases all the locks.

HTTP Session Handler: cl_http_server, line 5031
