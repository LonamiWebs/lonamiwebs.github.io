```meta
title: MongoDB: Operaciones Básicas y Arquitectura
published: 2020-03-05T03:00:53+00:00
updated: 2020-03-20T11:42:15+00:00
```

Este es el segundo post en la serie sobre MongoDB, con una breve descripción de las operaciones básicas (tales como inserción, recuperación e indexado), y ejecución por completo junto con el modelo de datos y arquitectura.

Otros posts en esta serie:

* [MongoDB: Introducción](/blog/mdad/mongodb-introduction/)
* [MongoDB: Operaciones Básicas y Arquitectura](/blog/mdad/mongodb-operaciones-basicas-y-arquitectura/) (este post)

Este post está hecho en colaboración con un compañero, y en él veremos algunos ejemplos de las operaciones básicas ([CRUD](https://stackify.com/what-are-crud-operations/)) sobre MongoDB.

----------

Empezaremos viendo cómo creamos una nueva base de datos dentro de MongoDB y una nueva colección donde poder insertar nuestros documentos.

## Creación de una base de datos e inserción de un primer documento

Podemos ver las bases de datos que tenemos disponibles ejecutando el comando:

```
> show databases
admin   0.000GB
config  0.000GB
local   0.000GB
```

Para crear una nueva base de datos, o utilizar una de las que tenemos creadas ejecutamos `use` junto con el nombre que le vamos a dar:

```
> use new_DB
switched to db new_DB
```

Una vez hecho esto, podemos ver que si volvemos a ejecutar «show databases», la nueva base de datos no aparece. Esto es porque para que Mongo registre una base de datos en la lista de las existentes, necesitamos insertar al menos un nuevo documento en una colección de esta. Lo podemos hacer de la siguiente forma:

```
> db.movie.insert({"name":"tutorials point"})
WriteResult({ "nInserted" : 1 })

> show databases
admin       0.000GB
config      0.000GB
local       0.000GB
movie       0.000GB
```

Al igual que podemos ver las bases de datos existentes, también podemos consultar las colecciones que existen dentro de estas. Siguiendo la anterior ejecución, si ejecutamos:

```
> show collections
movie
```

### Borrar base de datos

Para borrar una base de datos tenemos que ejecutar el siguiente comando:

```
> db.dropDatabase()
{ "dropped" : "new_DB", "ok" : 1 }
```

### Crear colección

Para crear una colección podemos hacerlo de dos formas. O bien mediante el comando:

```
db.createCollection(<nombre de la colección>, opciones)
```

Donde el primer parámetro es el nombre que le queremos asignar a la colección, y los siguientes, todos opcionales, pueden ser (entre otros):

<table class="">
 <thead>
  <tr>
   <th>
    Campo
   </th>
   <th>
    Tipo
   </th>
   <th>
    Descripción
   </th>
  </tr>
 </thead>
 <tbody>
  <tr>
   <td>
    <code>
     capped
    </code>
   </td>
   <td>
    Booleano
   </td>
   <td>
    Si es
    <code>
     true
    </code>
    ,
 permite una colección limitada. Una colección limitada es una colección
 de tamaño fijo que sobrescribe automáticamente sus entradas más 
antiguas cuando alcanza su tamaño máximo. Si especifica
    <code>
     true
    </code>
    , también debe especificar el parámetro de
    <code>
     size
    </code>
    .
   </td>
  </tr>
  <tr>
   <td>
    <code>
     autoIndexId
    </code>
   </td>
   <td>
    Booleano
   </td>
   <td>
    Si es
    <code>
     true
    </code>
    crea automáticamente un índice en el campo
    <code>
     _id
    </code>
    . Por defecto es
    <code>
     false
    </code>
   </td>
  </tr>
  <tr>
   <td>
    <code>
     size
    </code>
   </td>
   <td>
    Número
   </td>
   <td>
    Especifica el tamaño máximo en bytes para una colección limitada. Es obligatorio si el campo
    <code>
     capped
    </code>
    está a
    <code>
     true
    </code>
    .
   </td>
  </tr>
  <tr>
   <td>
    <code>
     max
    </code>
   </td>
   <td>
    Número
   </td>
   <td>
    Especifica el número máximo de documentos que están permitidos en la colección limitada.
   </td>
  </tr>
 </tbody>
</table>

```
> use test
switched to db test

> db.createCollection("mycollection")
{ "ok" : 1 }

> db.createCollection("mycol", {capped : true, autoIndexId: true, size: 6142800, max: 10000})
{
    "note" : "the autoIndexId option is deprecated and will be removed in a future release",
    "ok" : 1
}

> show collections
mycol
mycollection
```

Como se ha visto anteriormente al crear la base de datos, podemos insertar un documento en una colección sin que la hayamos creado anteriormente. Esto es porque MongoDB crea automáticamente una colección cuando insertas algún documento en ella:

```
> db.tutorialspoint.insert({"name":"tutorialspoint"})
WriteResult({ "nInserted" : 1 })

> show collections
mycol
mycollection
tutorialspoint
```

### Borrar colección

Para borrar una colección basta con situarnos en la base de datos que la contiene, y ejecutar lo siguiente:

```
db.<nombre_de_la_colección>.drop()
```

```
> db.mycollection.drop()
true

> show collections
mycol
tutorialspoint
```

### Insertar documento

Para insertar datos en una colección de MongoDB necesitaremos usar el método `insert()` o `save()`.

Ejemplo del método `insert`:

```
> db.colection.insert({
... title: 'Esto es una prueba para MDAD',
... description: 'MongoDB es una BD no SQL',
... by: 'Classmate and Me',
... tags: ['mongodb', 'database'],
... likes: 100
... })
WriteResults({ "nInserted" : 1 })
```

En este ejemplo solo se ha insertado un único documento, pero podemos insertar los que queramos separándolos de la siguiente forma:

```
db.collection.insert({documento}, {documento2}, {documento3})
```

No hace falta especificar un ID ya que el propio mongo asigna un ID a cada documento automáticamente, aunque nos da la opción de poder asignarle uno mediante el atributo `_id` en la inserción de los datos

Como se indica en el título de este apartado también se puede insertar mediante el método `db.coleccion.save(documento)`, funcionando este como el método `insert`.

### Método `find()`

El método find en MongoDB es el que nos permite realizar consultas a las colecciones de nuestra base de datos:

```
db.<nombre_de_la_colección>.find()
```

Este método mostrará de una forma no estructurada todos los documentos de la colección. Si le añadimos la función `pretty` a este método, se mostrarán de una manera más «bonita».

```
> db.colection.find()
{ "_id": ObjectId("5e738f0989f85a7eafdf044a"), "title" : "Esto es una prueba para MDAD", "description" : "MongoDB es una BD no SQL", "by" : "Classmate and Me", "tags" : [ "mongodb", "database" ], "likes" : 100 }

> db.colection.find().pretty()
{
    "_id": ObjectId("5e738f0989f85a7eafdf044a"),
    "title" : "Esto es una prueba para MDAD",
    "description" : "MongoDB es una BD no SQL",
    "by" : "Classmate and Me",
    "tags" : [
        "mongodb",
        "database"
    ],
    "likes" : 100
}
```

Los equivalentes del `where` en las bases de datos relacionales son:

<table class="">
 <thead>
  <tr>
   <th>
    Operación
   </th>
   <th>
    Sintaxis
   </th>
   <th>
    Ejemplo
   </th>
   <th>
    Equivalente en RDBMS
   </th>
  </tr>
 </thead>
 <tbody>
  <tr>
   <td>
    Igual
   </td>
   <td>
    <code>
     {&lt;clave&gt;:&lt;valor&gt;}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"by":"Classmate and Me"})
    </code>
   </td>
   <td>
    <code>
     where by = 'Classmate and Me'
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Menor que
   </td>
   <td>
    <code>
     {&lt;clave&gt;:{$lt:&lt;valor&gt;}}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"likes":{$lt:60}})
    </code>
   </td>
   <td>
    <code>
     where likes &lt; 60
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Menor o igual que
   </td>
   <td>
    <code>
     {&lt;clave&gt;:{$lte:&lt;valor&gt;}}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"likes":{$lte:60}})
    </code>
   </td>
   <td>
    <code>
     where likes &lt;= 60
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Mayor que
   </td>
   <td>
    <code>
     {&lt;clave&gt;:{$gt:&lt;valor&gt;}}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"likes":{$gt:60}})
    </code>
   </td>
   <td>
    <code>
     where likes &gt; 60
    </code>
   </td>
  </tr>
  <tr>
   <td>
    Mayor o igual que
   </td>
   <td>
    <code>
     {&lt;clave&gt;:{$gte:&lt;valor&gt;}}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"likes":{$gte:60}})
    </code>
   </td>
   <td>
    <code>
     where likes &gt;= 60
    </code>
   </td>
  </tr>
  <tr>
   <td>
    No igual
   </td>
   <td>
    <code>
     {&lt;clave&gt;:{$ne:&lt;valor&gt;}}
    </code>
   </td>
   <td>
    <code>
     db.mycol.find({"likes":{$ne:60}})
    </code>
   </td>
   <td>
    <code>
     where likes != 60
    </code>
   </td>
  </tr>
 </tbody>
</table>

En el método `find()` podemos añadir condiciones AND y OR de la siguiente manera:

```
(AND)
> db.colection.find({$and:[{"by":"Classmate and Me"},{"title": "Esto es una prueba para MDAD"}]}).pretty()

(OR)
> db.colection.find({$or:[{"by":"Classmate and Me"},{"title": "Esto es una prueba para MDAD"}]}).pretty()

(Ambos a la vez)
> db.colection.find({"likes": {$gt:10}, $or: [{"by": "Classmate and Me"}, {"title": "Esto es una prueba para MDAD"}]}).pretty()
```

La última llamada con ambos a la vez equivalente en una consulta SQL a:

```
where likes>10 AND (by = 'Classmate and Me' OR title = 'Esto es una prueba para MDAD')
```

### Actualizar un documento

En MongoDB se hace utilizando el método `update`:

```
db.<nombre_colección>.update(<criterio_de_selección>, <dato_actualizado>)
```

Para este ejemplo vamos a actualizar el documento que hemos insertado en el apartado anterior:

```
> db.colection.update({'title':'Esto es una prueba para MDAD'},{$set:{'title':'Título actualizado'}})
WriteResult({ "nMatched" : 1, "nUpserted" : 0, "nModified" : 1 })
> db.colection.find().pretty()
{
    "_id": ObjectId("5e738f0989f85a7eafdf044a"),
    "title" : "Título actualizado",
    "description" : "MongoDB es una BD no SQL",
    "by" : "Classmate and Me",
    "tags" : [
        "mongodb",
        "database"
    ],
    "likes" : 100
}
```

Anteriormente se ha mencionado el método `save()` para la inserción de documentos, pero también podemos utilizarlo para sustituir documentos enteros por uno nuevo:

```
> db.<nombre_de_la_colección>.save({_id:ObjectId(), <nuevo_documento>})
```

Con nuestro documento:

```
> db.colection.save(
...   {
...     "_id": ObjectId("5e738f0989f85a7eafdf044a"), "title": "Este es el nuevo título", "by": "MDAD"
...   }
... )
WriteResult({ "nMatched" : 1, "nUpserted" : 0, "nModified" : 1 })

> db.colection.find()
{
    "_id": ObjectId("5e738f0989f85a7eafdf044a"),
    "title": "Este es el nuevo título",
    "by": "MDAD"
}
```

### Borrar documento

Para borrar un documento utilizaremos el método `remove()` de la siguiente manera:

```
db.<nombre_de_la_colección>.remove(<criterio_de_borrado>)
```

Considerando la colección del apartado anterior borraremos el único documento que tenemos:

```
> db.colection.remove({'title': 'Este es el nuevo título'})
WriteResult({ "nRemoved" : 1 })
> db.colection.find().pretty()
>
```

Para borrar todos los documentos de una colección usamos:

```
db.<colección>.remove({})
```

### Indexación

MongDB nos permite crear índices sobre atributos de una colección de la siguiente forma:

```
db.<colección>.createIndex( {<atributo>:<opciones>})
```

Como ejemplo:

```
> db.mycol.createIndex({"title":1})
{
    "createdCollectionAutomatically" : false,
    "numIndexesBefore" : 1,
    "numIndexesAfter" : 2,
    "ok" : 1
}
```

Si queremos más de un atributo en el índice lo haremos así:

```
> db.mycol.ensureIndex({"title":1,"description":-1})
```

Los valores que puede tomar son `+1` para ascendente o `-1` para descendente.

### Referencias

* Manual MongoDB. (n.d.). [https://docs.mongodb.com/manual/](https://docs.mongodb.com/manual/)
* MongoDB Tutorial – Tutorialspoint. (n.d.). – [https://www.tutorialspoint.com/mongodb/index.htm](https://www.tutorialspoint.com/mongodb/index.htm)
