class KWayMerge:
    def keyLessThan(this, src, pa, a, b) -> bool:
        cmp = compareValues(src[pa[a].readInt()], src[pa[b].readInt()])
        return cmp < 0 or (cmp == 0 and a < b)

    def siftDown(this, src, heap, pa, t, r, size):
        while 2*r+2 < size:
            nxt = 2*r+1
            min_ = nxt+int(not this.keyLessThan(src, pa,
                           heap[nxt].readInt(), heap[nxt+1].readInt()))
            if this.keyLessThan(src, pa, heap[min_].readInt(), t):
                heap[r].write(heap[min_])
                r = min_
            else:
                break
        min_ = 2*r+1
        if min_ < size and this.keyLessThan(src, pa, heap[min_].readInt(), t):
            heap[r].write(heap[min_])
            r = min_
        heap[r].write(t)

    def kWayMerge(this, src, dest, heap, pa, pb, size):
        i = 0
        while i < size:
            heap[i].write(i)
            i += 1
        i = (size-1)//2
        while i >= 0:
            this.siftDown(src, heap, pa, heap[i].readInt(), i, size)
            i -= 1
        i = 0
        while size > 0:
            min_ = heap[0].readInt()
            dest[i].write(src[pa[min_].readInt()])
            pa[min_] += 1
            if pa[min_] == pb[min_]:
                size -= 1
                this.siftDown(src, heap, pa, heap[size].readInt(), 0, size)
            else:
                this.siftDown(src, heap, pa, heap[0].readInt(), 0, size)
            i += 1

kWayMerge = KWayMerge()