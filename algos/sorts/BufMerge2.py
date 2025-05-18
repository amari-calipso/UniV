class BufMerge2:
    RUN_SIZE = 32

    def __init__(this, rot):
        if rot is None:
            this.rotate = UniV_getUserRotation("Select rotation algorithm (default: Helium)").indexed
        else:
            this.rotate = sortingVisualizer.getRotationByName(rot).indexed

    def sortRuns(this, array, a, b):
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*4))
        i = a
        while i < b-this.RUN_SIZE:
            binaryInsertionSort(array, i, i+this.RUN_SIZE)
            i += this.RUN_SIZE
        if i < b:
            binaryInsertionSort(array, i, b)
        sortingVisualizer.setSpeed(speed)

    def mergeInPlaceBW(this, array, a, m, b):
        rotate = this.rotate
        s = b-1
        l = m-1
        while s > l and l >= a:
            if array[l] > array[s]:
                p = lrBinarySearch(array, a, l, array[s], False)
                rotate(array, p, l+1, s+1)
                s -= l+1-p
                l = p-1
            else:
                s -= 1

    def mergeWithStatBufferBW(this, array, a0, b0, a1, b1, buf):
        l = b0-1
        r = b1-1
        o = buf-1
        while l >= a0 and r >= a1:
            if array[r] >= array[l]:
                array[o].swap(array[r])
                r -= 1
            else:
                array[o].swap(array[l])
                l -= 1
            o -= 1
        while r >= a1:
            array[o].swap(array[r])
            o -= 1
            r -= 1
        while l >= a0:
            array[o].swap(array[l])
            o -= 1
            l -= 1

    def gallopMerge(this, array, a0, b0, a1, b1, buf):
        l = b0-1
        r = b1-1
        o = buf-1
        while l >= a0 and r >= a1:
            if array[l] > array[r]:
                k = lrBinarySearch(array, a0, l, array[r], False)
                while l >= k:
                    array[l].swap(array[o])
                    l -= 1
                    o -= 1
            array[r].swap(array[o])
            r -= 1
            o -= 1
        while r >= a1:
            array[o].swap(array[r])
            o -= 1
            r -= 1
        while l >= a0:
            array[o].swap(array[l])
            o -= 1
            l -= 1

    def mergeWithScrollingBufferFW(this, array, a, m, b):
        o = a-(m-a)
        l = a
        r = m
        while l < m and r < b:
            if array[l] <= array[r]:
                array[o].swap(array[l])
                l += 1
            else:
                array[o].swap(array[r])
                r += 1
            o += 1
        while l < m:
            array[o].swap(array[l])
            o += 1
            l += 1
        while r < b:
            array[o].swap(array[r])
            o += 1
            r += 1

    def mergeWithScrollingBufferBW(this, array, a, m, b):
        l = m-1
        r = b-1
        o = r+m-a
        while r >= m and l >= a:
            if array[r] >= array[l]:
                array[o].swap(array[r])
                r -= 1
            else:
                array[o].swap(array[l])
                l -= 1
            o -= 1
        while r >= m:
            array[o].swap(array[r])
            o -= 1
            r -= 1
        while l >= a:
            array[o].swap(array[l])
            o -= 1
            l -= 1

    def buildFW(this, array, a, b):
        s = a
        e = b
        r = this.RUN_SIZE
        while r < b-a:
            twoR = 2*r
            i: int
            i = s
            while i < e-twoR:
                this.mergeWithScrollingBufferFW(array, i, i+r, i+twoR)
                i += twoR
            if i+r < e:
                this.mergeWithScrollingBufferFW(array, i, i+r, e)
            s -= r
            e -= r
            r = twoR

    def buildBW(this, array, a, b):
        s = a
        e = b
        r = this.RUN_SIZE
        while r < b-a:
            twoR = 2*r
            i: int
            i = e
            while i >= s+twoR:
                this.mergeWithScrollingBufferBW(array, i-twoR, i-r, i)
                i -= twoR
            if i-r >= s:
                this.mergeWithScrollingBufferBW(array, s, i-r, i)
            s += r
            e += r
            r = twoR

    def sortBuf(this, array, a, b):
        n = b-a
        if n <= this.sqrtn:
            binaryInsertionSort(array, a, b)
            return -1
        h = n//2-(n & 1)
        a += this.RUN_SIZE
        this.sortRuns(array, a, a+h)
        this.buildBW(array, a, a+h)
        return a+h-this.RUN_SIZE

    def sort(this, array, a, b):
        n = b-a
        h: int
        if n <= this.RUN_SIZE:
            binaryInsertionSort(array, a, b)
            return
        sqrtn = 1
        while sqrtn*sqrtn < n:
            sqrtn *= 2
        this.sqrtn = sqrtn
        gallop = n//log2(n)
        h = n//2+(n & 1)-this.RUN_SIZE
        b -= this.RUN_SIZE
        this.sortRuns(array, a+h, b)
        this.buildFW(array, a+h, b)
        b += this.RUN_SIZE
        s = a+h+this.RUN_SIZE
        while True:
            p = this.sortBuf(array, s, b)
            if p == -1:
                this.mergeInPlaceBW(array, a, s, b)
                return
            if b-p > gallop:
                this.mergeWithStatBufferBW(array, a, s, p, b, p)
            else:
                this.gallopMerge(array, a, s, p, b, p)
            s = p


@Sort("Merge Sorts", "In-Place Buffered Merge Sort II", "Buf Merge 2")
def bufMerge2Run(array):
    BufMerge2(None).sort(array, 0, len(array))
