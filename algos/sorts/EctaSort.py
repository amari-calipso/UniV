class EctaSort:
    def __init__(this):
        this.tags = None

    def mergeTo(this, from_, to, a, m, b, p):
        i = a
        j = m
        while i < m and j < b:
            if from_[i] <= from_[j]:
                to[p].write(from_[i])
                i += 1
            else:
                to[p].write(from_[j])
                j += 1
            p += 1
        while i < m:
            to[p].write(from_[i])
            p += 1
            i += 1
        while j < b:
            to[p].write(from_[j])
            p += 1
            j += 1

    def pingPongMerge(this, array, buf, a, m1, m2, m3, b):
        p = 0
        p1 = p+m2-a
        pEnd = p+b-a
        this.mergeTo(array, buf, a, m1, m2, p)
        this.mergeTo(array, buf, m2, m3, b, p1)
        this.mergeTo(buf, array, p, p1, pEnd, a)

    def mergeBWExt(this, array, tmp, a, m, b):
        s = b-m
        arrayCopy(array, m, tmp, 0, s)
        i = s-1
        j = m-1
        while i >= 0 and j >= a:
            b -= 1
            if tmp[i] >= array[j]:
                array[b].write(tmp[i])
                i -= 1
            else:
                array[b].write(array[j])
                j -= 1
        while i >= 0:
            b -= 1
            array[b].write(tmp[i])
            i -= 1

    def blockCycle(this, array, buf, keys, a, bLen, bCnt):
        i = 0
        while i < bCnt:
            if keys[i] != i:
                arrayCopy(array, a+i*bLen, buf, 0, bLen)
                j = i
                next = keys[i].readInt()
                while True:
                    arrayCopy(array, a+next*bLen, array, a+j*bLen, bLen)
                    keys[j].write(j)
                    j = next
                    next = keys[next].readInt()
                    if not (next != i):
                        break
                arrayCopy(buf, 0, array, a+j*bLen, bLen)
                keys[j].write(j)
            i += 1

    def blockMerge(this, array, buf, tags, a, m, b, bLen):
        c = 0
        t = 2
        i = a
        j = m
        k = 0
        l = 0
        r = 0
        while c < 2*bLen:
            if array[i] <= array[j]:
                buf[k].write(array[i])
                i += 1
                l += 1
            else:
                buf[k].write(array[j])
                j += 1
                r += 1
            k += 1
            c += 1
        left = l >= r
        k = i-l if left else j-r
        c = 0
        while True:
            if i < m and (j == b or array[i] <= array[j]):
                array[k].write(array[i])
                i += 1
                l += 1
            else:
                array[k].write(array[j])
                j += 1
                r += 1
            k += 1
            c += 1
            if c == bLen:
                tags[t].write((k-a)//bLen-1)
                t += 1
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
        arrayCopy(array, k-c, array, b1, c)
        r -= c
        t = 0
        k = 0
        while l > 0:
            arrayCopy(buf, k, array, m-l, bLen)
            tags[t].write((m-a-l)//bLen)
            t += 1
            k += bLen
            l -= bLen
        while r > 0:
            arrayCopy(buf, k, array, b1-r, bLen)
            tags[t].write((b1-a-r)//bLen)
            t += 1
            k += bLen
            r -= bLen
        this.blockCycle(array, buf, tags, a, bLen, (b-a)//bLen)

    def sort(this, array, a, b):
        if b-a <= 32:
            binaryInsertionSort(array, a, b)
            return
        bLen = 1
        while bLen*bLen < b-a:
            bLen *= 2
        tLen = (b-a)//bLen
        bufLen = 2*bLen
        j = 16
        speed = sortingVisualizer.getSpeed()
        sortingVisualizer.setSpeed(max(int(10*(len(array)/2048)), speed*2))
        i = a
        while i < b:
            binaryInsertionSort(array, i, min(i+j, b))
            i += j
        sortingVisualizer.setSpeed(speed)
        buf = sortingVisualizer.createValueArray(bufLen)
        tags = sortingVisualizer.createValueArray(tLen)
        sortingVisualizer.setNonOrigAux([tags])
        while 4*j <= bufLen:
            i = a
            while i+2*j < b:
                this.pingPongMerge(array, buf, i, i+j, i+2*j,
                                   min(i+3*j, b), min(i+4*j, b))
                i += 4*j
            if i+j < b:
                this.mergeBWExt(array, buf, i, i+j, b)
            j *= 4
        while j <= bufLen:
            i = a
            while i+j < b:
                this.mergeBWExt(array, buf, i, i+j, min(i+2*j, b))
                i += 2*j
            j *= 2
        while j < b-a:
            i = a
            while i+j+bufLen < b:
                this.blockMerge(array, buf, tags, i, i+j, min(i+2*j, b), bLen)
                i += 2*j
            if i+j < b:
                this.mergeBWExt(array, buf, i, i+j, b)
            j *= 2


@Sort("Block Merge Sorts", "Ecta Sort", "Ecta Sort")
def ectaSortRun(array):
    EctaSort().sort(array, 0, len(array))
