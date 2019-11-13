  $ . "${TEST_FIXTURES}/library.sh"

Setup config repo:
  $ setup_common_config
  $ cd "$TESTTMP"

Setup testing repo for mononoke:
  $ hg init repo-hg
  $ cd repo-hg
  $ setup_hg_server

Helper for making commit:
  $ function commit() { # the arg is used both for commit message and variable name
  >   hg commit -Am $1 # create commit
  >   export COMMIT_$1="$(hg --debug id -i)" # save hash to variable
  > }

First two simple commits and bookmark:
  $ echo -e "a\nb\nc\nd\ne" > a
  $ commit A
  adding a

  $ echo -e "a\nb\nd\ne\nf" > b
  $ commit B
  adding b
  $ hg bookmark -i BOOKMARK_B

A commit with a file change and binary file

  $ echo -e "b\nc\nd\ne\nf" > b
  $ echo -e "\0 10" > binary
  $ commit C
  adding binary

Commit with globalrev:
  $ touch c
  $ hg add
  adding c
  $ hg commit -Am "commit with globalrev" --extra global_rev=9999999999
  $ hg bookmark -i BOOKMARK_C

import testing repo to mononoke
  $ cd ..
  $ blobimport repo-hg/.hg repo --has-globalrev

try talking to the server before it is up
  $ SCS_PORT=$(get_free_socket) scsc lookup --repo repo  -B BOOKMARK_B
  error: apache::thrift::transport::TTransportException: AsyncSocketException: connect failed, type = Socket not open, errno = 111 (Connection refused): Connection refused
  [1]

start SCS server
  $ start_and_wait_for_scs_server

repos
  $ scsc repos
  repo

diff
  $ scsc diff --repo repo -B BOOKMARK_B -i $COMMIT_C
  diff --git a/b b/b
  --- a/b
  +++ b/b
  @@ -1,5 +1,5 @@
  -a
   b
  +c
   d
   e
   f
  diff --git a/binary b/binary
  new file mode 100644
  Binary file binary has changed

lookup using bookmarks
  $ scsc lookup --repo repo  -B BOOKMARK_B
  323afe77a1b1e632e54e8d5a683ba2cc8511f299

lookup, commit without globalrev
  $ scsc lookup --repo repo  -B BOOKMARK_B -S bonsai,hg,globalrev
  bonsai=c63b71178d240f05632379cf7345e139fe5d4eb1deca50b3e23c26115493bbbb
  hg=323afe77a1b1e632e54e8d5a683ba2cc8511f299

lookup, commit with globalrev
  $ scsc lookup --repo repo -B BOOKMARK_C -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using bonsai to identify commit
  $ scsc lookup --repo repo --bonsai-id 006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using globalrev to identify commit
  $ scsc lookup --repo repo --globalrev 9999999999 -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using hg to identify commit
  $ scsc lookup --repo repo --hg-commit-id ee87eb8cfeb218e7352a94689b241ea973b80402 -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using bonsai needed resolving to identify commit
  $ scsc lookup --repo repo -i 006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using globalrev needed resolving to identify commit
  $ scsc lookup --repo repo -i 9999999999 -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402

lookup using hg needed resolving to identify commit
  $ scsc lookup --repo repo -i ee87eb8cfeb218e7352a94689b241ea973b80402 -S bonsai,hg,globalrev
  bonsai=006c988c4a9f60080a6bc2a2fff47565fafea2ca5b16c4d994aecdef0c89973b
  globalrev=9999999999
  hg=ee87eb8cfeb218e7352a94689b241ea973b80402
