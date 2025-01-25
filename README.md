# Websocket1CAddIn
Внешняя компонента клиент для  работы с websocket

## Реквизиты объекта

#### ОписаниеОшибки

Текст последней ошибки любого из методов

## Методы

Все функции выбрасывают исключение, если произошла какая-нибудь ошибка.

#### `Подключиться(Адрес)`

Установка соединения с сервером

Возвращаемое значение:
- Булево - подключение установлено

#### `ОтправитьСообщение(Сообщение)`

Возвращаемое значение:
- Булево - сообщение отправлено


#### `ПолучитьСообщение(Таймаут = 1000)`

Таймаут задается в милисекундах (по умолчанию 1000). Во время выполнения функции текущий поток ждет получение сообщения от сервера и если не дожидается в течение заданного таймаута - возвращает пустую строку.

Возвращаемое значение:
- Строка - текстовое сообщение от сервера

#### `Отключиться()`

Отключение соединения

## Пример:

```1c-enterprise
ОбъектВК = Новый("AddIn.WebSocket1C.WebSocket1CAddIn");	
	
Попытка 
		
	СоединениеУстановлено = ОбъектВК.Подключиться("ws://127.0.0.1:8080"); 
    Таймаут = 2000;

	Если СоединениеУстановлено Тогда
			
        СообщениеОтправлено = ОбъектВК.ОтправитьСообщение("Hello World!");
        Сообщить("Результат отправки сообщения: " + СообщениеОтправлено);

	    Ответ = ОбъектВК.ПолучитьСообщение(Таймаут);
        Если ЗначениеЗаполнено(Ответ) Тогда 
			Сообщить("Сообщение от сервера: " + Ответ);
        Иначе
            Сообщить(СтрШаблон("Сервер не ответил в течение %1 милисекунд!", Таймаут));
        КонецЕсли;  

        ОбъектВК.Отключиться();

	КонецЕсли;	
			
Исключение
		
	Сообщить(ОбъектВК.ОписаниеОшибки);
		
КонецПопытки;
```
