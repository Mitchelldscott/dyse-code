/********************************************************************************
 * 
 *      ____                     ____          __           __       _          
 *	   / __ \__  __________     /  _/___  ____/ /_  _______/ /______(_)__  _____
 *	  / / / / / / / ___/ _ \    / // __ \/ __  / / / / ___/ __/ ___/ / _ \/ ___/
 *	 / /_/ / /_/ (__  )  __/  _/ // / / / /_/ / /_/ (__  ) /_/ /  / /  __(__  ) 
 *	/_____/\__, /____/\___/  /___/_/ /_/\__,_/\__,_/____/\__/_/  /_/\___/____/  
 *	      /____/                                                                
 * 
 * 
 * 
 ********************************************************************************/

#include <Arduino.h>
#include "utilities/blink.h"

#ifndef SYS_GRAPH_VECTOR
#define SYS_GRAPH_VECTOR

/*
	Class for lists of objects, the goal is to
	provide a buffer with less type and memory
	restrictions.
*/
template <typename T> class Vector {
	private:
		int length;
		int items;
		T* buffer;

	public:

		Vector() {
			length = 0;
			buffer = NULL;
		}

		~Vector() {
			if (buffer != NULL) {
				delete buffer;
			}
		}

		Vector(int size) {
			/*
				  Constructor for Vector with length = size.
				@param:
					size: (int) length of the buffer with type T
			*/
			items = 0;
			length = size;
			buffer = new T[length];
			clear();
		}

		Vector(T* data, int size) {
			/*
				  Constructor for Vector with length = size and non-zero data.
				@param:
					data: (T*) data to fill buffer with
					size: (int) length of the buffer with type T
			*/
			items = size;
			length = size;
			buffer = new T[length];
			memcpy(buffer, data, size * sizeof(T));
		}

		///// modifiers /////

		void clear() {
			/*
				  Clear data in the buffer (set to 0).
			*/
			if (buffer != NULL) {
				memset(buffer, 0, length * sizeof(T));
				items = 0;
			}
		}

		void reset(int size) {
			/*
				  Resize buffer and set data to zero.
				@param:
					data: (T*) data to fill buffer with
					size: (int) length of the buffer with type T
			*/
			if (buffer != NULL) {
				delete buffer;
			}

			length = size;
			buffer = new T[size];
			clear();
		}

		void resize(int size) {
			/*
				  Resize buffer and keep data.
				@param:
					size: (int) length of the buffer with type T
			*/
			size = uint8_t(size);

			if (length == size) {
				return;
			}

			T* tmp = new T[size];

			if (buffer != NULL) {
				if (length > size) {
					memcpy(tmp, buffer, size * sizeof(T));
					if (items > size) {
						items = size;
					}
				}
				else {
					memcpy(tmp, buffer, length * sizeof(T));
				}

				delete buffer;
			}

			length = size;
			buffer = tmp;
		}

		void make_even_fit(int n) {
			if (n > length) {
				length = max(1, length);
				int k = int(((n + 1) / 2.0)) * 2;
				resize(k);
			}
		}

		void set_items(int i) {
			items = i;
		}

		void push(T item) {
			/*
				  Add a single item T to the buffer.
				Basically and append wraper. (maybe rename this)
				@param:
					item: (T) data to add to buffer
			*/
			if (items >= length) {
				if (length == 0) {
					length = 1;
				}
				resize(2 * length);
			}

			buffer[items] = item;
			items += 1;
		}

		void from_array(T* data, int size) {
			/*
				  reset the buffer to size n with data T*.
				@param:
					data: (T*) data to fill buffer with
					size: (int) length of the buffer with type T
			*/
			make_even_fit(size);
			memcpy(buffer, data, size * sizeof(T));
			items = size;
		}

		void append(T* data, int n) {
			/*
				  Add n values to the buffer. Stores the current buffer
				calls reset and then copies buffers into resized buffer.
				@param:
					data: (T*) data to fill buffer with
					size: (int) length of the buffer with type T
			*/
			make_even_fit(items + n);
			memcpy(&buffer[items], data, n * sizeof(T));
			items += n;
		}

		void append(Vector<T> data) {
			/*
				  Add n values to the buffer. Stores the current buffer and
				data to add a temp. Calls reset and then copies buffer into resized buffer.
				@param:
					data: (Vector<T>*) data to fill buffer with
			*/
			int n = data.size();
			make_even_fit(items + n);
			memcpy(&buffer[items], data.as_array(), n * sizeof(T));
			items += n;
		}

		void append(Vector<T>* data) {
			/*
				  Add n values to the buffer. Stores the current buffer and
				data to add a temp. Calls reset and then copies buffer into resized buffer.
				@param:
					data: (Vector<T>*) data to fill buffer with
			*/
			int n = data->size();
			make_even_fit(items + n);
			memcpy(&buffer[items], data->as_array(), n * sizeof(T));
			items += n;
		}

		void insert(T* data, int index, int n) {
			/*
				  Add n values to the buffer starting at index. If buffer is not
				large enough it will be extended. This works much better if
				the vector is already large enough (append can call reset).
				@param:
					data: (T*) data to fill buffer with
					index: (int) index to start insertion
					n: (int) number of items to insert
			*/
			make_even_fit(index + n);
			memcpy(&buffer[index], data, n * sizeof(T));
			items = index + n;
		}

		void insert(Vector<T> data, int index) {
			/*
				  Add n values to the buffer starting at index. If buffer is not
				large enough it will be extended. This works much better if
				the vector is already large enough (append can call reset).
				@param:
					data: (Vector<T>) data to fill buffer with
					index: (int) index to start insertion
			*/
			int n = data.size();
			make_even_fit(index + n);
			memcpy(&buffer[index], data.as_array(), n * sizeof(T));
			items = index + n;
		}

		///// accessors /////

		int size() {
			/*
				  Get the size of buffer (not necessarily elements available)
				@return
					length: (int) size of buffer
			*/
			return items;
		}

		int len() {
			/*
				  Get the size of buffer (not necessarily elements available)
				@return
					length: (int) size of buffer
			*/
			return length;
		}

		int find(T data) {
			/*
				  Get the first index of an item in the buffer
				@return
					data: (T) item to search for (-1 if not found, maybe causes template issues)
			*/
			for (int i = 0; i < items; i++) {
				if (buffer[i] == data) {
					return i;
				}
			}
			return -1;
		}

		T* as_array() {
			return buffer;
		}

		void slice(T* data, int start, int n) {
			/*
				  Get a slice of the buffer, must have valid indices.
				@return
					data: (T*) buffer to put slice
					start: (int) start index of buffer
					n: (int) number of items in slice
			*/
			if (start >= 0 && start + n <= length && buffer != NULL) {
				memcpy(data, &buffer[start], n * sizeof(T));
			}
			else {
				memset(data, 0, n * sizeof(T));
			}
		}

		T pop() {
			T tmp[items];
			T popee = buffer[0];
			slice(tmp, 1, items-1);
			insert(tmp, 0, items);
			items = max(0, items - 1);
			return popee;
		}

		///// operators /////

		T& operator[](int index) {
			/*
				  [] Operator overload
				@param
					index: (int) index of item in buffer to access
				@return
					item: (T&) item at index
				@exit
					when index is invalid 
			*/
			if (index < 0) {
				index = max(0, length + index);
			}

			items = max(items, index+1);

			if (length > index) {
				return buffer[index];
			}
			else {
				make_even_fit(index);
				return buffer[index];
			}
		}

		void operator=(Vector<T>* data) {
			/*
				  = Operator overload. Will reset this vector to the
				same size as data.
				@param
					data: (Vector<T>*) data to copy
			*/
			from_array(data->as_array(), data->len());
		}

		void operator=(Vector<T>& data) {
			/*
				  = Operator overload. Will reset this vector to the
				same size as data.
				@param
					data: (Vector<T>&) data to copy
			*/
			from_array(data.as_array(), data.len());
		}

		void print();
};

#endif