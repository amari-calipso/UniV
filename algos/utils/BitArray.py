class BitArray:
    def __init__(this, array, pa, pb, size, w):
        this.array = array
        this.pa = pa
        this.pb = pb
        this.size = size
        this.w = w
        this.length = size*w

    def flipBit(this, a, b):
        this.array[a].swap(this.array[b])

    def getBit(this, a, b):
        return this.array[a] > this.array[b]

    def setBit(this, a, b, bit):
        if this.getBit(a, b) ^ bit:
            this.flipBit(a, b)

    def free(this):
        i1 = this.pa+this.length
        i = this.pa
        j = this.pb
        while i < i1:
            this.setBit(i, j, False)
            i += 1
            j += 1

    def set(this, idx, uInt):
        assert idx >= 0 and idx < this.size
        s = idx*this.w
        i1 = this.pa+s+this.w
        i = this.pa+s
        j = this.pb+s
        while i < i1:
            this.setBit(i, j, (uInt & 1) == 1)
            i += 1
            j += 1
            uInt >>= 1
        if uInt > 0:
            print("Warning: Word too large\n")

    def setXor(this, idx, val):
        assert idx >= 0 and idx < this.size
        s = idx*this.w
        i1 = this.pa+s+this.w
        i = this.pa+s
        j = this.pb+s
        while i < i1:
            if (val & 1) == 1:
                this.flipBit(i, j)
            i += 1
            j += 1
            val >>= 1
        if val > 0:
            print("Warning: Word too large\n")

    def get(this, idx):
        assert idx >= 0 and idx < this.size
        r = 0
        s = idx*this.w
        k = 0
        i = this.pa+s
        j = this.pb+s
        while k < this.w:
            r |= int(this.getBit(i, j)) << k
            k += 1
            i += 1
            j += 1
        return r

    def incr(this, idx):
        assert idx >= 0 and idx < this.size
        s = idx*this.w
        i1 = this.pa+s+this.w
        i = this.pa+s
        j = this.pb+s
        while i < i1:
            this.flipBit(i, j)
            if this.getBit(i, j):
                return
            i += 1
            j += 1
        print("Warning: Integer overflow\n")

    def decr(this, idx):
        assert idx >= 0 and idx < this.size
        s = idx*this.w
        i1 = this.pa+s+this.w
        i = this.pa+s
        j = this.pb+s
        while i < i1:
            this.flipBit(i, j)
            if not this.getBit(i, j):
                return
            i += 1
            j += 1
        print("Warning: Integer underflow\n")

    def swap(this, a, b):
        assert a >= 0 and a < this.size
        assert b >= 0 and b < this.size
        tmp = this.get(a)
        this.set(a, this.get(b))
        this.set(b, tmp)
