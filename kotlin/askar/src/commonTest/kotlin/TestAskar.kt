package org.hyperledger.askar_uniffi

import askar_uniffi.AskarEntryOperation
import askar_uniffi.AskarKeyAlg
import askar_uniffi.AskarSession
import askar_uniffi.AskarStore
import askar_uniffi.AskarStoreManager
import askar_uniffi.Disposable
import askar_uniffi.LocalKeyFactory
import kotlinx.coroutines.runBlocking
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull


class TestAskar {

    private lateinit var store: AskarStore
    private lateinit var session: AskarSession
    private lateinit var storeManager: AskarStoreManager
    private lateinit var keyFactory: LocalKeyFactory
    private lateinit var ffiObjects: MutableList<Disposable>
    
    private val testEntry = buildMap {
        put("category", "test category")
        put("name", "test name")
        put("value", "test_value")
        put("tags", "{\"~plaintag\": \"a\", \"enctag\": \"b\"}")
    }
    private val uriSchema = "sqlite://${getDBDirectory()}"

    @BeforeTest
    fun beforeEach(){
        runBlocking{
            storeManager = AskarStoreManager()
            keyFactory = LocalKeyFactory()

            val key = storeManager.generateRawStoreKey(null)
            store = storeManager.provision("${uriSchema}test.db", "raw", key, null, true)

            ffiObjects = mutableListOf(store, storeManager, keyFactory)
        }
    }

    @AfterTest
    fun afterEach(){
        runBlocking{
            session.closeSession()
            try{
                store.closeStore()
            }catch(e: Throwable){
                println("Failed to close the store")
                println(e.message)
            }
            try{
                storeManager.remove("${uriSchema}test.db")
            }catch(e: Throwable){
                println("Failed to delete store db")
                println(e.message)
            }
        }
        ffiObjects.forEach{
            it.destroy()
        }
    }


    @Test
    fun testStoreClose() {
        runBlocking {
            session = store.session(null)
            ffiObjects.add(session)
            val count = session.count("test", null)
            assertEquals(0, count)
        }
    }

    @Test
    fun testInsertUpdate(){
        runBlocking {
            session = store.session(null)
            ffiObjects.add(session)
            session.update(
                AskarEntryOperation.INSERT,
                testEntry["category"]!!,
                testEntry["name"]!!,
                testEntry["value"]!!.encodeToByteArray(),
                testEntry["tags"]!!,
                null
            )

            val count = session.count(
                testEntry["category"]!!,
                "{\"~plaintag\": \"a\", \"enctag\": \"b\"}"
            )
            assertEquals(1, count)

            var found = session.fetch(
                testEntry["category"]!!,
                testEntry["name"]!!,
                false
            )?: throw Error("Entry not found")
            ffiObjects.add(found)

            assertEquals(testEntry["category"], found.category())
            assertEquals(testEntry["name"], found.name())
            assertEquals(testEntry["value"], found.value().decodeToString())
            val tags = found.tags()
            assertEquals("a", tags["plaintag"])
            assertEquals("b", tags["enctag"])

            val all = session.fetchAll(
                testEntry["category"]!!,
                "{\"~plaintag\": \"a\", \"enctag\": \"b\"}",
                null,
                false
            )
            all.forEach{
                ffiObjects.add(it)
            }
            assertEquals(1, all.size)

            val first = all[0]
            assertEquals(testEntry["name"], first.name())
            assertEquals(testEntry["value"], first.value().decodeToString())

            val newEntry = testEntry.toMutableMap()
            newEntry["value"] = "new value"
            newEntry["tags"] = "{\"upd\": \"tagval\"}"
            session.update(
                AskarEntryOperation.REPLACE,
                newEntry["category"]!!,
                newEntry["name"]!!,
                newEntry["value"]!!.encodeToByteArray(),
                newEntry["tags"]!!,
                null
            )

            found = session.fetch(
                newEntry["category"]!!,
                newEntry["name"]!!,
                false
            )?: throw Error("Entry not found")
            ffiObjects.add(found)
            assertEquals(newEntry["value"], found.value().decodeToString())
            assertEquals("tagval", found.tags()["upd"])

            session.update(
                AskarEntryOperation.REMOVE,
                testEntry["category"]!!,
                testEntry["name"]!!,
                byteArrayOf(),
                null,
                null
            )

            val empty = session.fetch(
                testEntry["category"]!!,
                testEntry["name"]!!,
                false
            )
            if (empty != null) {
                ffiObjects.add(empty)
            }

            assertNull(empty)
        }
    }

    @Test
    fun testScan(){
        runBlocking {
            session = store.session(null)
            ffiObjects.add(session)
            session.update(
                AskarEntryOperation.INSERT,
                testEntry["category"]!!,
                testEntry["name"]!!,
                testEntry["value"]!!.encodeToByteArray(),
                testEntry["tags"]!!,
                null
            )

            val scan = store.scan(
                null,
                testEntry["category"]!!,
                "{\"~plaintag\": \"a\", \"enctag\": \"b\"}",
                null,
                null
            )
            ffiObjects.add(scan)

            val rows = scan.fetchAll()
            rows.forEach{
                ffiObjects.add(it)
            }

            assertEquals(1, rows.size)
            val first = rows[0]
            assertEquals(testEntry["name"], first.name())
            assertEquals(testEntry["value"], first.value().decodeToString())
        }
    }

    @Test
    fun testKeyStore(){
        runBlocking {
            session = store.session(null)
            ffiObjects.add(session)
            val keypair = keyFactory.generate(AskarKeyAlg.ED25519, false)
            ffiObjects.add(keypair)
            val keyName = "test_key"
            session.insertKey(keyName, keypair, "metadata", "{\"a\": \"b\"}", null)

            var key = session.fetchKey(keyName, false)
            ffiObjects.add(key!!)
            assertEquals(keyName, key.name())
            assertEquals("b", key.tags()["a"])

            session.updateKey(keyName, "new metadata", "{\"a\": \"c\"}", null)
            key = session.fetchKey(keyName, false)
            ffiObjects.add(key!!)
            assertEquals(keyName, key.name())
            assertEquals("c", key.tags()["a"])

            val thumbprint = keypair.toJwkThumbprint(null)
            assertEquals(thumbprint, key.loadLocalKey().toJwkThumbprint(null))

            val keylist = session.fetchAllKeys(
                "ed25519",
                thumbprint,
                "{\"a\": \"c\"}",
                -1,
                false
            )
            keylist.forEach {
                ffiObjects.add(it)
            }
            assertEquals(1, keylist.size)
            assertEquals(keyName, keylist[0].name())
        }
    }

}