Useful resources found poking around in the ADT Plugin Source, analyzing network traffic and testing endpoints.


# Stateful / Stateless Analysis

Whether the session is stateful or statless is determined through the `X-sap-adt-sessiontype` header, e.g `stateful` or `stateless`.

Eclipse seems to create a stateful session for the first time when you lock an object and then uses that same session for all following stateful requests.

Transaction SM04 allows you to view the sessions, it also shows that Eclipse still uses RFC Sessions
Transaction SM12 allows to see what locks a user has in place

`sap-contextid` is the cookie responsible for managing the session data

