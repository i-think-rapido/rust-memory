# Memory implementation

It is essentially a shared memory hash table with a time-to-live parameter.

You should periodically call the `forget()` method to perform garbage collection.
