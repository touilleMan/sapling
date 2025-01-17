# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License found in the LICENSE file in the root
# directory of this source tree.
#require slow

  $ . "${TEST_FIXTURES}/library.sh"

setup configuration
  $ setup_common_config
  $ mononoke_testtool drawdag -R repo --derive-all <<'EOF'
  > A-B-C
  >    \
  >     D
  > # bookmark: C main
  > EOF
  A=aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675
  B=f8c75e41a0c4d29281df765f39de47bca1dcadfdc55ada4ccc2f6df567201658
  C=e32a1e342cdb1e38e88466b4c1a01ae9f410024017aa21dc0a1c5da6b3963bf2
  D=5a25c0a76794bbcc5180da0949a652750101597f0fbade488e611d5c0917e7be

derived-data exists:

Simple usage
  $ mononoke_newadmin derived-data -R repo exists -T unodes -i aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675
  Derived: aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675
Multiple changesets
  $ mononoke_newadmin derived-data -R repo exists -T unodes -i aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675 -i 5a25c0a76794bbcc5180da0949a652750101597f0fbade488e611d5c0917e7be
  Derived: aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675
  Derived: 5a25c0a76794bbcc5180da0949a652750101597f0fbade488e611d5c0917e7be
Bookmark
  $ mononoke_newadmin derived-data -R repo exists -T unodes -B main
  Derived: e32a1e342cdb1e38e88466b4c1a01ae9f410024017aa21dc0a1c5da6b3963bf2

derived-data count-underived:

Simple usage
  $ mononoke_newadmin derived-data -R repo count-underived -T unodes -i aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675
  aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675: 0
Multiple changesets
  $ mononoke_newadmin derived-data -R repo count-underived -T unodes -i aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675 -i 5a25c0a76794bbcc5180da0949a652750101597f0fbade488e611d5c0917e7be
  aa53d24251ff3f54b1b2c29ae02826701b2abeb0079f1bb13b8434b54cd87675: 0
  5a25c0a76794bbcc5180da0949a652750101597f0fbade488e611d5c0917e7be: 0
Bookmark
  $ mononoke_newadmin derived-data -R repo count-underived -T unodes -B main
  e32a1e342cdb1e38e88466b4c1a01ae9f410024017aa21dc0a1c5da6b3963bf2: 0

