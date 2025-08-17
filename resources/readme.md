### Useful stuff found poking around in the ADT Plugin, analyzing traffic and testing endpoints.

> [!IMPORTANT]
> SAP provides little documentation about alot of the related concepts, especially the internal handling of it.
> Most of the stuff written down here comes out of personal research with an occasional link to official documentation.
> This also implies that information here is prone to not be fully correct.

# Statefulness
Stateful and stateless are terms that will often come up in the context of communicating with SAP endpoints. Some public info:
- [Stateful or Stateless Programming]
- [Stateless/Stateful Communication]

## What is it used for
Stateful requests are required to perform operations on the sap system that rely on context across multiple requests. The most common example for this 
is modifying objects. To modify an object, you must first lock it for the duration of your modifications.

Its important to understand that, while you might be tempted to call it a stateful session, it is more suitable to call it a context/transaction within the session.
A HTTP session on the server is denoted by the `SAP_SESSIONID_<SYSTEM>_<CLIENT>` cookie. You can observe these in transaction `SM05`. These are the [Security Sessions]
sessions that will expire if no request is made on them in some timeframe.
> `http_dispatch_request` @1187

Contexts are stored in the `sap-contextid` Cookie and only exist in the scope of the session they were created in. If you try to use the context in a different
session, you will get an error. When the session ends, either due to the user explicitly logging off or the timeout expiring, all the associated contexts are 
taken down with it. SAP calls these 'User Sessions' - you can observe these in `SM04 `. The crux is that each of these also occupies a work process. If you keep
too many open at once, you will severely impact the performance of your system.

## ADT as an example
To modify Objects on the system, you must first lock the resource and then unlock it afterwards. This can be done through the corresponding endpoint, e.g
`<object>?_action=LOCK&accessMode=MODIFY`. 

If you do this without a stateful transaction and observe your locks in SM12, you will notice that no persistent lock is created. 
If you attempt to update the object using the handle obtained from the previous request, you receive an error that the lock is not valid.

The reason for this should be apparent. Locks serve the purpose to prevent multiple users from editing an object at once. They should only exist for the duration
of the context that a user is actually editing the object. If the user forgets to unlock the object, or the client fails to do so, the object would otherwise remain locked.

Thus the locks are bound to a context, which in itself is also bound to a timeout. When the session expires, so do the associated locks. This ensures no objects remain locked unintentionally.
Whether the session is stateful or statless is determined through the `X-sap-adt-sessiontype` header: `stateful` or `stateless`.

## Other useful information
Transaction SM12 to see the locks on objects by user
Log off at `/sap/public/bc/icf/logoff` which invalidates your session & contexts, thus releasing all the locks.

SESSIONID Cookie is Security Session Cookie, see fn module http_dispatch_request start=1187
https://help.sap.com/docs/SAP_INTEGRATED_BUSINESS_PLANNING/685fbd2d5f8f4ca2aacfc35f1938d1c1/c7379ecf6a8f4c0bb09e88142124c77f.html

[Security Sessions]: https://help.sap.com/docs/SAP_INTEGRATED_BUSINESS_PLANNING/685fbd2d5f8f4ca2aacfc35f1938d1c1/c7379ecf6a8f4c0bb09e88142124c77f.html
[Stateful or Stateless Programming]: https://help.sap.com/doc/saphelp_ewm900/9.0/en-us/4c/5b00dd980a7514e10000000a42189b/content.htm?no_cache=true
[Stateless/Stateful Communication]: https://help.sap.com/doc/saphelp_em92/9.2/en-US/48/d1853df6c96745e10000000a421937/content.htm?no_cache=true
