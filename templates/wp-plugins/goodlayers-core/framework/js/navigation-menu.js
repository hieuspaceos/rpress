(function($){
	"use strict";
	
	// ref : http://unscriptable.com/2009/03/20/debouncing-javascript-methods/
	// ensure 1 is fired
	window.gdlr_core_debounce = function(func, threshold, execAsap){
		
		var timeout;

		return function debounced(){
			
			var obj = this, args = arguments;
			
			function delayed(){
				if( !execAsap ){
					func.apply(obj, args);
				}
				timeout = null;
			};

			if( timeout ){
				clearTimeout(timeout);
			}else if( execAsap ){
				func.apply(obj, args);
			}
			timeout = setTimeout(delayed, threshold);
		};
	}	
	
	$(document).ready(function(){
		
		$('#menu-management').on('input change gdlr-core-change', '[data-slug]', function(){
			var parent_option = $(this).closest('.gdlr-core-custom-nav-menu-fields');
			var obj = {};
			parent_option.find('[data-slug]').each(function(){
				if( $(this).is('input[type="checkbox"]') ){
					obj[$(this).attr('data-slug')] = ($(this).is(':checked'))? 'enable': 'disable';
				}else{
					obj[$(this).attr('data-slug')] = $(this).val();
				}
			});
			console.log(obj);
			parent_option.children('.gdlr-core-custom-nav-menu-val').val(JSON.stringify(obj));
		});

		// add colorpicker
		var menu_cp = $('#menu-to-edit');
		setInterval(function(){
			menu_cp.children().not('.gdlr-core-colorpicker-added').each(function(){
				$(this).addClass('gdlr-core-colorpicker-added');
				$(this).find('.gdlr-core-colorpicker').each(function(){
					var wpcp = $(this);
					$(this).wpColorPicker({
						change: gdlr_core_debounce(function(event, ui){
							wpcp.trigger('gdlr-core-change');
						}, 500),
						clear: gdlr_core_debounce(function(){
							wpcp.trigger('gdlr-core-change');
						}, 500)
					});
				});
			});
		}, 1000);

		$('#menu-management').on('click', '.gdlr-core-upload-button', function(){
			var input_image = $(this).siblings('.gdlr-core-upload-input-img');
			var input_val = $(this).siblings('.gdlr-core-upload-input');
			var remove_button = $(this).siblings('.gdlr-core-upload-button-remove');

			var frame = wp.media({
				multiple: false
			}).on('select', function(){
	  
				// Get media attachment details from the frame state
				var attachment = frame.state().get('selection').first().toJSON();

				input_image.attr('src', attachment.url);
				input_val.val(attachment.id).trigger('change');
				remove_button.addClass('gdlr-core-active');
			}).open();
		});
		$('#menu-management').on('click', '.gdlr-core-upload-button-remove', function(){
			var input_image = $(this).siblings('.gdlr-core-upload-input-img');
			var input_val = $(this).siblings('.gdlr-core-upload-input');

			input_image.attr('src', '');
			input_val.val('').trigger('change');
			$(this).removeClass('gdlr-core-active');
		});
	});
	
})(jQuery);	