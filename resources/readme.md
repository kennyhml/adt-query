Useful resources found poking around in the ADT Plugin Source, analyzing network traffic and testing endpoints.


# Stateful / Stateless Analysis

Whether the session is stateful or statless is determined through the `X-sap-adt-sessiontype` header, e.g `stateful` or `stateless`.

Why are stateful sessions needed?
We can make a request to lock an object, e.g `POST ../adt/oo/classes/Z_TEST?_action=LOCK&accessMode=MODIFY` without the `stateful` header.
We do get back a seemingly fine 200 OK with the lock handle and some additional information, but when you observe SM12, there actually is no lock.
This is also proven when you try to update the resource using the handle, e.g `POST ../adt/oo/classes/Z_TEST/source/main?lockHandle=...` you get back a 423 (Resource Locked). 
Which is a bizarre return code considering the error information in the body then informs you that the resource is, in fact, not locked and the handle is not valid :D

And the reason is simple. The lock is created, but only for the duration of the session. In other words, once the session (the request) ends, the lock is dropped again.
This is why such operations need to be made `stateful`, we rely on context / data persistence across calls.

Eclipse seems to create a stateful session for the first time when you lock an object and then uses that same session for all following stateful requests.

Transaction SM04 allows you to view the sessions, it also shows that Eclipse still uses RFC Sessions
Transaction SM12 allows to see what locks a user has in place

`sap-contextid` is the cookie responsible for managing the session data

`sap/bc/adt/core/http/sessions` shows session handling information, NOT ACTIVE SESSIONS! You can also log off at `/sap/public/bc/icf/logoff`
