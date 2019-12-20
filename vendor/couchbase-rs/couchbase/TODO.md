# Overall
    - Move to sdk3 error handling structure
    - Remove all expect code and turn into errors based on the structure

# KV
    - implement options for get and the 3 mutation options
    - add durability requirement options (+ sync durability)
    - remaining commands plus options:
        - replica get
        - mutateIn
            fullDocument
            arrayAppend
            arrayPrepend
            arrayInsert
            arrayAddUnique
            increment
            decrement
    - Binary collection
        - append
        - prepend
        - increment
        - decrement

# Querying
    - add all query options for
        * n1ql
        * analytics
    - add views plus options
    - add fts plus options

# Testing
    - set up and run basic testing infra with mock and real node