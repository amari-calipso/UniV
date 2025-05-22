class KotaSort:
    def tLenCalc(this, n, bLen):
        n1 = n-2*bLen
        a = 0
        b = 2*bLen
        m: int
        while a < b:
            m = (a+b)//2
            if n1-m < (m+3)*bLen:
                b = m
            else:
                a = m+1
        return a

    def findKeysBW(this, array, a, b, nKeys):
        rotate = this.rotate
        f = 1
        p = b-f
        loc: int
        i = p
        while i > a and f < nKeys:
            loc = lrBinarySearch(array, p, p+f, array[i-1], True)-p
            if loc == f or array[i-1] < array[p+loc]:
                rotate(array, i, p, p+f)
                f += 1
                p = i-1
                rotate(array, i-1, i, p+loc+1)
            i -= 1
        rotate(array, p, p+f, b)
        return f

    def mergeTo(this, array, a, m, b, p):
        i = a
        j = m
        while i < m and j < b:
            if array[i] <= array[j]:
                array[p].swap(array[i])
                i += 1
            else:
                array[p].swap(array[j])
                j += 1
            p += 1
        while i < m:
            array[p].swap(array[i])
            p += 1
            i += 1
        while j < b:
            array[p].swap(array[j])
            p += 1
            j += 1

    def pingPongMerge(this, array, a, m1, m, m2, b, p):
        p1 = p+m-a
        pEnd = p+b-a
        this.mergeTo(array, a, m1, m, p)
        this.mergeTo(array, m, m2, b, p1)
        this.mergeTo(array, p, p1, pEnd, a)

    def inPlaceMergeBW(this, array, a, m, b):
        rotate = this.rotate
        while b > m and m > a:
            i = lrBinarySearch(array, a, m, array[b-1], False)
            rotate(array, i, m, b)
            t = m-i
            m = i
            b -= t+1
            if m == a:
                break
            b = lrBinarySearch(array, m, b, array[m-1], True)

    def selectMin(this, array, a, b, bLen):
        min = a
        i = min+bLen
        while i < b:
            if array[i] < array[min]:
                min = i
            i += bLen
        return min

    def blockSelect(this, array, a, b, t, bLen):
        while a < b:
            min = this.selectMin(array, a, b, bLen)
            if min != a:
                blockSwap(array, a, min, bLen)
            array[a].swap(array[t])
            t += 1
            a += bLen

    def blockMerge(this, array, a, m, b, t, p, bLen):
        c = 0
        tp = t
        i = a
        j = m
        k = p
        l = 0
        r = 0
        while c < 2*bLen:
            if array[i] <= array[j]:
                array[k].swap(array[i])
                i += 1
                l += 1
            else:
                array[k].swap(array[j])
                j += 1
                r += 1
            k += 1
            c += 1
        left = l >= r
        k = i-l if left else j-r
        c = 0
        while True:
            if i < m and (j == b or array[i] <= array[j]):
                array[k].swap(array[i])
                i += 1
                l += 1
            else:
                array[k].swap(array[j])
                j += 1
                r += 1
            k += 1
            c += 1
            if c == bLen:
                sortingVisualizer.delay(250)
                array[k-bLen].swap(array[tp])
                tp += 1
                if left:
                    l -= bLen
                else:
                    r -= bLen
                left = l >= r
                k = i-l if left else j-r
                c = 0
            if not (i < m or j < b):
                break
        b1 = b-c
        blockSwap(array, k-c, b1, c)
        r -= c
        blockSwap(array, m-l, a, l)
        blockSwap(array, b1-r, a+l, r)
        blockSwap(array, a, p, 2*bLen)
        this.blockSelect(array, a+2*bLen, b1, t, bLen)

    def blockMergeNoBuf(this, array, a, m, b, t, bLen):
        i = a+bLen
        j = t
        while i < m:
            sortingVisualizer.delay(250)
            array[i].swap(array[j])
            i += bLen
            j += 1
        i = a+bLen
        b1 = b-(b-m) % bLen
        while i < m and m < b1:
            if array[i-1] > array[m+bLen-1]:
                blockSwap(array, i, m, bLen)
                this.inPlaceMergeBW(array, a, i, i+bLen)
                m += bLen
            else:
                min = this.selectMin(array, i, m, bLen)
                if min > i:
                    blockSwap(array, i, min, bLen)
                array[t].swap(array[i])
                t += 1
            i += bLen
        if i < m:
            while True:
                min = this.selectMin(array, i, m, bLen)
                if min > i:
                    blockSwap(array, i, min, bLen)
                array[t].swap(array[i])
                t += 1
                i += bLen
                if not (i < m):
                    break
        else:
            while m < b1 and array[m-1] > array[m]:
                this.inPlaceMergeBW(array, a, m, m+bLen)
                m += bLen
        this.inPlaceMergeBW(array, a, b1, b)

    def sort(this, array, a, b):
        if b-a <= 32:
            binaryInsertionSort(array, a, b)
            return
        bLen = 1
        while bLen*bLen < b-a:
            bLen *= 2
        tLen = this.tLenCalc(b-a, bLen)
        bufLen = 2*bLen
        j = 16
        keys = this.findKeysBW(array, a, b, bufLen+tLen)
        if keys == 1:
            return
        elif keys <= 8:
            i = a
            while i < b:
                binaryInsertionSort(array, i, min(i+j, b))
                i += j
            while j < b-a:
                i = a
                while i+j < b:
                    this.inPlaceMergeBW(array, i, i+j, min(i+2*j, b))
                    i += 2*j
                j *= 2
            return
        if keys < bufLen+tLen:
            while bufLen > 2*(keys-bufLen):
                bufLen //= 2
            bLen = bufLen//2
            tLen = keys-bufLen
        b1 = b-keys
        t = b1
        p = b1+tLen
        i = a
        while i < b1:
            binaryInsertionSort(array, i, min(i+j, b1))
            i += j
        while 4*j <= bufLen:
            i = a
            while i+2*j < b1:
                this.pingPongMerge(array, i, i+j, i+2*j,
                                   min(i+3*j, b1), min(i+4*j, b1), p)
                i += 4*j
            if i+j < b1:
                HeliumSort__mergeWithBufferBW(None, array, i, i+j, b1, p, True)
            j *= 4
        while j <= bufLen:
            i = a
            while i+j < b1:
                HeliumSort__mergeWithBufferBW(None, array, i, i+j, min(i+2*j, b1), p, True)
                i += 2*j
            j *= 2
        limit = bLen*(tLen+3)
        while j < b1-a and min(2*j, b1-a) < limit:
            i = a
            while i+j+bufLen < b1:
                this.blockMerge(array, i, i+j, min(i+2*j, b1), t, p, bLen)
                i += 2*j
            if i+j < b1:
                HeliumSort__mergeWithBufferBW(None, array, i, i+j, b1, p, True)
            j *= 2
        binaryInsertionSort(array, p, b)
        if bufLen <= tLen:
            bufLen *= 2
        bLen = 2*j//bufLen
        while j < b1-a:
            i = a
            while i+j+2*bLen < b1:
                this.blockMergeNoBuf(array, i, i+j, min(i+2*j, b1), t, bLen)
                i += 2*j
            if i+j < b1:
                this.inPlaceMergeBW(array, i, i+j, b1)
            j *= 2
            bLen *= 2
        this.inPlaceMergeBW(array, a, b1, b)


if __name__ == '__main__':
    kotaSort = KotaSort()
    kotaSort.rotate = sortingVisualizer.getRotationByName("Cycle Reverse").indexed


@Sort("Block Merge Sorts", "Kota Sort", "Kota Sort")
def kotaSortRun(array):
    rotate = UniV_getUserRotation("Select rotation algorithm (default: Cycle Reverse)").indexed
    oldRotate = kotaSort.rotate
    kotaSort.rotate = rotate
    kotaSort.sort(array, 0, len(array))
    kotaSort.rotate = oldRotate
