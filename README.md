# tifsep - The incredibly fast search engine proxy

tifsep ia a metasearch engine that provides very fast search results
by working on the individual packets of the search results instead
of slowly waiting, parsing, validating, sorting, and then finally
sending the results to the user.

Even without HTTP/3 support currently*, tifsep can yield results within
300ms, which is faster than most search engines.

*HTTP/3 support is coming soon!

**tifsep is currently in early development.**
If you'd like to test it out and provide early feedback, please do so!

## Ideas

* HTTP3 support
* First show all results, then sort them on server, hide old results and show new sorted ones
* Check if results have already been sent and don't send them again
