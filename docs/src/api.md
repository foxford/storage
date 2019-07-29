**Storage** provides various APIs to access stored content
- Both **Object and Set** are the most convenient APIs for clients. In order to receive content, clients don’t need to perform any additional steps, they just follow redirect (default behavior for clients like browsers).
    The only difference between those two APIs is an authorization object of a request:
    - for **Object API** each particular object within bucket and set need to be authorized
    - for **Set API** only bucket and set need to be authorized, that allows more efficient authorization with leaser number of requests to external authorization server
- **Sign API** is the most flexible API. A client receives a signed URI in payload of a response, then it's the responsibility of the client to retrieve content by using the obtained URI. Sign API may also be used to perform update and delete actions along with read action. Note that the signed URI has expiration time.
