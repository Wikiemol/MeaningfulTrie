Meaningfulness Trie Explorer
===================================

![](./images/welcome2.gif)

This project allows you to explore a special kind of doubly linked suffix trie. 

The suffix trie allows you to approximate certain information theoretical properties of a text file.

Each node stores 

- the maximum depth in the trie that that subtrie exists in the trie. 
- the frequency that the string represented by that node has been seen (this string is the concatenation of the path from the root to this node).

Running the project
---------------------

It is a Rust cargo project. Once you have rust and cargo installed, simply clone the repository, and run 

`cargo install`

then 

`cargo run`

Navigation
--------------

- `l` to go right
- `h` to go left
- `j` to go down
- `k` to go up
- Enter to expand a node
- Tab to go to the longest meaningful string (explained in the theory section below)


Commands
------------

To type a command, type `:<command>`

- [`:load`](#load-command) load a file from the file system
- [`:meaning`](#meaningfulness) display meaningfulness of all nodes


Theory
----------

If `l` is the depth of the node, and `f` is the frequency, then `l * log(f)` is defined to be the 'meaningfulness' of that string. The meaningfulness roughly tells you how 'important' the string is. 

High meaningfulness corresponds to the 'completeness' of a phrase. E.g. the string 'complet' is less meaningful than the string 'complete'. Because complete, is, well... complete. Moreover, when you go further than a complete phrase the meaningfulness drops. So this allows for an algorithm that finds complete words and phrases without prior knowledge of a given language, purely through information theory.

In the other direction, when you see a phrase like 'omplete', the 'max depth' in the trie will be higher than its actual depth. So you can detect that this phrase is not complete in this way, and travel to its maximum depth parent. This is what [pressing tab](#tab-navigation) does.

If you take the ratio  of the meaningfulness of a phrase in a specific document to the meaningfulness of that phrase over all documents, this is the 'relative meaningfulness' of that phrase. The relative meaningfulness of the phrase is how descriptive that phrase is of the document. 

Images
------------

### Load command 
![](./images/load.gif)
### Meaningfulness 
![](./images/meaningfulness.gif)
### Tab navigation 
![](./images/Tab.gif)
