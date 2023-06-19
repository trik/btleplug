package com.nonpolynomial.btleplug.droidplug


class ScanFilter(uuids: Array<String?>?) {
    private var uuids: Array<String?> = arrayOfNulls(0);

    init {
        if (uuids != null) {
            val len = uuids.size
            this.uuids = uuids.copyOf(len)
        }
    }

    fun getUuids(): Array<String?> {
        val len = uuids.size
        return uuids.copyOf(len)
    }
}
