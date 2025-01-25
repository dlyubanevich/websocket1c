# Websocket1CAddIn
Внешняя компонента клиент для  работы с websocket

## Реквизиты объекта

#### ОписаниеОшибки

Текст последней ошибки любого из методов

## Методы

Все функции выбрасывают исключение, если произошла какая-нибудь ошибка.

### Подключиться(Адрес)

Установка соединения с сервером

Возвращаемое значение:
- Булево - подключение установлено

### ОтправитьСообщение(Сообщение)

Возвращаемое значение:
- Булево - сообщение отправлено


### ПолучитьСообщение()

Во время выполнения функции текущий поток висит и ждет получение сообщения от сервера.

Возвращаемое значение:
- Строка - текстовое сообщение от сервера

### Отключиться()

Отключение соединения

## Пример:

```1c-enterprise
ОбъектВК = Новый("AddIn.WebSocket1C.WebSocket1CAddIn");	
	
Попытка 

	СоединениеУстановлено = ОбъектВК.Подключиться("ws://127.0.0.1:8080"); 
	
	Если СоединениеУстановлено Тогда
		СообщениеОтправлено = ОбъектВК.ОтправитьСообщение("Hello World!");
		Ответ = ОбъектВК.ПолучитьСообщение();
		Сообщить("ПолучитьСообщение: " + Ответ);
		ОбъектВК.Отключиться()
	КонецЕсли;	
			
Исключение
		
	Сообщить(ОбъектВК.ОписаниеОшибки);
		
КонецПопытки;
```
