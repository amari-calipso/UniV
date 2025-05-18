class AdjacencyPancakeSort:
    def dualSwap(this, array, a, b):
        this.keys[a].swap(this.keys[b])
        array[a].swap(array[b])

    def reversal(this, array, a, b):
        while b-a > 1:
            b -= 1
            this.dualSwap(array, a, b)
            a += 1

    def isAdjacent(this, a, b, N):
        return this.keys[b] == (this.keys[a].readInt()+1) % N or this.keys[a] == (this.keys[b].readInt()+1) % N

    def findAdjacent(this, e, a, N):
        while not this.isAdjacent(a, e, N):
            a += 1
        return a

    def sort(this, array, a, b):
        N = b-a
        if N == 2:
            if array[a] > array[a+1]:
                reverse(array, a, a+2)
            return
        this.keys = sortingVisualizer.createValueArray(N)
        sortingVisualizer.setNonOrigAux([this.keys])
        j = a
        while j < b:
            c = 0
            i = a
            while i < b:
                if i == j:
                    i += 1
                    continue
                cmp = compareValues(array[i], array[j])
                if cmp < 0 or (cmp == 0 and i < j):
                    c += 1
                i += 1
            this.keys[j-a].write(c)
            j += 1
        while True:
            i = a
            while i < b-1 and this.isAdjacent(i, i+1, N):
                i += 1
            if i == b-1:
                break
            if i == a:
                j = this.findAdjacent(a, a+2, N)
                if not this.isAdjacent(j-1, j, N):
                    this.reversal(array, a, j)
                else:
                    k = this.findAdjacent(a, j+1, N)
                    if not this.isAdjacent(k-1, k, N):
                        this.reversal(array, a, k)
                    else:
                        this.reversal(array, a, j+1)
                        this.reversal(array, a, j)
                        this.reversal(array, a, k+1)
                        this.reversal(array, a, a+k-j)
            else:
                j = this.findAdjacent(a, i+1, N)
                if not this.isAdjacent(j-1, j, N):
                    this.reversal(array, a, j)
                else:
                    k = this.findAdjacent(i, i+2, N)
                    if k+1 < b and this.isAdjacent(k+1, k, N):
                        this.reversal(array, a, i+1)
                        this.reversal(array, a, k+1)
                    elif this.isAdjacent(k-1, k, N):
                        this.reversal(array, a, k+1)
                        this.reversal(array, a, a+k-i)
                    else:
                        this.reversal(array, a, k+1)
                        this.reversal(array, a, a+k-i)
                        if j < k:
                            this.reversal(array, a, k+1)
                            this.reversal(array, a, i+k-j+1)
                        else:
                            this.reversal(array, a, j+1)
                            this.reversal(array, a, a+j-k)
        i = a
        while this.keys[i] != 0 and this.keys[i] != N-1:
            i += 1
        if this.keys[i] == 0:
            if i == a:
                return
            this.reversal(array, a, b)
            i = b-2-(i-a)
        elif i == a:
            this.reversal(array, a, b)
            return
        i += 1
        this.reversal(array, a, i)
        this.reversal(array, a, b)
        this.reversal(array, a, b-i)


@Sort("Pancake Sorts", "Adjacency Pancake Sort", "Adjacency Pancake")
def adjacencyPancakeSortRun(array):
    AdjacencyPancakeSort().sort(array, 0, len(array))
