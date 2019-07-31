# API

- **Set and Tag APIs** are convenient way to retrieve content, clients don’t need to perform any additional steps, they just follow redirect (default behavior for clients like browsers).
    - **Set API** is used to access content by its **location in underlying backend**.
    - **Tag API** is used to create tags for sets and then use them to **categorize and authorize content** differently without a need of creating any copies of that content.
- With **Sign API** clients may perform **update and delete actions** along with read action. Note that the signed URI retrieved with the API has expiration time.
