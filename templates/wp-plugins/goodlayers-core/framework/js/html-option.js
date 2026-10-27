// for page option
(function($){
	"use strict";

	$(document).ready(function(){
		$('body.profile-php, body.user-edit-php, body.edit-tags-php, body.term-php').each(function(){
			var html_option = new gdlrCoreHtmlOption($(this));
		});
		
		$('.gdlr-core-page-option-content').each(function(){
			
			var data_input = $(this).siblings('.gdlr-core-page-option-value');
			var html_option = new gdlrCoreHtmlOption($(this));
			
			$('#post-preview, #publish, #save-post, .editor-post-publish-button, .editor-post-preview').click(function(){
				data_input.val(JSON.stringify(html_option.get_val()));
			});
			$('#wpbody-content').on('click', '.editor-header__settings, .edit-post-header__settings, .editor-post-publish-button, .editor-post-preview, .editor-post-save-draft', function(){
				data_input.val(JSON.stringify(html_option.get_val()));
			});

			// action for tab
			var tab_head = $(this).find('#gdlr-core-page-option-tab-head');
			var tab_content = $(this).find('#gdlr-core-page-option-tab-content');
			tab_head.children('.gdlr-core-page-option-tab-head-item').click(function(){
				if( $(this).hasClass('gdlr-core-active') ){ return; }
				
				var active_tab = $(this).attr('data-tab-slug');
				$(this).addClass('gdlr-core-active').siblings().removeClass('gdlr-core-active');
				tab_content.find('[data-tab-slug="' + active_tab + '"]').fadeIn(200).siblings().css('display', 'none');
			});
			
			// ajax save
			var security = $(this).siblings('[name="page_option_security"]').val();
			$(this).siblings('.gdlr-core-page-option-head').on('click', '.gdlr-core-page-option-head-save', function(){

				if( $(this).hasClass('gdlr-core-now-loading') ) return; 

				var nav_update_button = $(this).addClass('gdlr-core-now-loading');
				var post_id = $(this).attr('data-post-id');

				data_input.val(JSON.stringify(html_option.get_val()));
				
				// nonce
				$.ajax({
					type: 'POST',
					url: $(this).attr('data-ajax-url'),
					data: { 'security': security, 'action': $(this).attr('data-ajax-action'), 'post_id': post_id, 'name': data_input.attr('name'), 'value': data_input.val() },
					dataType: 'json',
					error: function(jqXHR, textStatus, errorThrown){
						nav_update_button.removeClass('gdlr-core-now-loading');
						gdlr_core_alert_box({ status: 'failed', head: nav_update_button.attr('data-failed-head'), message: nav_update_button.attr('data-failed-message') });
						
						// for displaying the debug text
						console.log(jqXHR, textStatus, errorThrown);
					},
					success: function(data){

						nav_update_button.removeClass('gdlr-core-now-loading');
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});	
					}
				});	
			});

		}); // gdlr-core-page-option-content
		
	}); // document ready
	
})(jQuery);	

// html option
(function($){
	"use strict";

	// use to init tmce element
	function gdlr_core_remove_unused_tmce_element(){
		if( typeof(tinymce) != 'undefined' ){
			window.gdlr_core_tmce_id_removed
			for( var i = 1; i < window.gdlr_core_tmce_id; i++ ){
				var tmce_id = 'gdlr_core_tmce' + i;

				var temp_tmce = tinymce.get(tmce_id);
				if( temp_tmce ){
					if( $('#wp-' + tmce_id + '-wrap').length == 0 ){
						temp_tmce.remove();
					}
				}
			}
		}
	}
	function gdlr_core_init_tmce_element( obj ){
		if( typeof(gdlr_core_tmce) == 'undefined' ) return;

		if( typeof(window.gdlr_core_tmce_id) == 'undefined' ){
			window.gdlr_core_tmce_id = 1;
		}else{
			window.gdlr_core_tmce_id++;
		}

		gdlr_core_remove_unused_tmce_element();

		var tmce_id = 'gdlr_core_tmce' + window.gdlr_core_tmce_id;
		obj.attr('data-tmce-id', tmce_id);

		var temp_tmce = gdlr_core_tmce.replace(/html-active/g, 'tmce-active');
		temp_tmce = temp_tmce.replace(/gdlr_core_tmce/g, tmce_id);

		var tmce_dom = $(temp_tmce);
		tmce_dom.insertAfter(obj);

		var content = obj.html();
		if( content.length > 0 ){
			if(tmce_dom.hasClass('tmce-active')){
				tmce_dom.find('textarea').val(content);
			}else if(tmce_dom.hasClass('html-active')){
				if( typeof(window.switchEditors) != 'undefined' ){
					tmce_dom.find('textarea').val(window.switchEditors._wp_Nop(content));
				}else{
					tmce_dom.find('textarea').val(gdlr_core_wp_Nop(content));
				}
				
			}
		}

		// init tinymce element
		if( (typeof(tinymce) !== 'undefined') && (typeof(tinyMCEPreInit.mceInit['gdlr_core_tmce']) !== 'undefined') ){
			var mceInit = JSON.parse(JSON.stringify(tinyMCEPreInit.mceInit['gdlr_core_tmce']));
			mceInit.selector = '#' + tmce_id;
			mceInit.body_class = mceInit.body_class.replace(/gdlr_core_tmce/g, tmce_id);
			mceInit.setup = function(ed){
				ed.on('change', function(e){
					$(e.target.targetElm).trigger('gdlr_core_change');
				});
			}
			var tmce_wrap = tinymce.$('#wp-' + tmce_id + '-wrap'); 
			if( (tmce_wrap.hasClass('tmce-active') || !tinyMCEPreInit.qtInit.hasOwnProperty(tmce_id)) && !mceInit.wp_skip_init ) {
				if( tmce_wrap.hasClass('html-active') ){
					tmce_wrap.removeClass('html-active').addClass('tmce-active');
				}
				tinymce.init(mceInit);
			}
		}

		// init quicktag
		if( typeof quicktags !== 'undefined' ){
			var qtInit = JSON.parse(JSON.stringify(tinyMCEPreInit.qtInit['gdlr_core_tmce']));
			qtInit.id = tmce_id;

			quicktags(qtInit);
			QTags._buttonsInit();
		}
	}
	function gdlr_core_get_tmce_data( obj ){
		var tmce_id = obj.attr('data-tmce-id');

		if( typeof(tinymce) != 'undefined' ){
			if(tinymce.$('#wp-' + tmce_id + '-wrap').hasClass('tmce-active')){
				return tinymce.get(tmce_id).getContent();
			}else if(tinymce.$('#wp-' + tmce_id + '-wrap').hasClass('html-active')){
				if( typeof(window.switchEditors) != 'undefined' ){
					return window.switchEditors._wp_Autop($('#' + tmce_id).val());
				}else{
					return gdlr_core_autop($('#' + tmce_id).val());
				}
			}
		}else{
			return gdlr_core_wp_Nop($('#' + tmce_id).val());
		}

		return '';
	}

	// html option
	window.gdlrCoreHtmlOption = function( container ){
		
		this.container = $(container);

		this.icon_list_target = null;
		this.icon_list_selection = '';

		this.init();
	}
	
	gdlrCoreHtmlOption.prototype = {
		
		// bind the action and events
		init: function(){
			
			// bind input format
			this.bind_input_format();

			// bind the image uploader
			this.bind_image_uploader();
						
			// bind the font picker
			this.bind_font_picker();
			
			// bind the icon selector
			this.bind_icon_selector();
			
			// bind conditional for showing item
			this.bind_conditional();
			
			// bind the fixed event item
			this.rebind();
		},
		
		// rebind specific element when content in the container changed
		rebind: function(){
			
			var t = this;
			
			t.container.find('.gdlr-core-html-option-colorpicker').wpColorPicker();

			t.init_conditional();
			
			t.bind_tinymce();
			
			t.bind_font_slider();

			t.bind_import();

			t.bind_export();
			
			t.bind_datepicker();

			// bind custom item
			t.container.find('[data-type="custom"]').each(function(){
				var custom_val = $(this).children('.gdlr-core-html-option-custom-value').data('value');
				var custom_options = $(this).children('.gdlr-core-html-option-custom-options').data('value');
				
				if( custom_val ){
					$(this).data('value', custom_val);
				}
				if( custom_options ){
					$(this).data('options', custom_options);	
				}

				if( $(this).is('[data-item-type="skin-settings"]') ){
					new gdlr_core_skin_settings($(this));
				}else if( $(this).is('[data-item-type="fontupload"]') ){
					new gdlr_core_font_upload($(this));
				}else if( $(this).is('[data-item-type="thumbnail-sizing"]') ){
					new gdlr_core_thumbnail_sizing($(this));
				}else if( $(this).is('[data-item-type="tabs"]') ){
					new gdlr_core_tabs($(this));
				}else if( $(this).is('[data-item-type="gallery"]') ){
					new gdlr_core_gallery($(this));
				}else if( $(this).is('[data-item-type="padding"]') ){
					new gdlr_core_padding($(this));
				}else if( $(this).is('[data-item-type="key-value"]') ){
					new gdlr_core_key_value($(this));
				}else if( $(this).is('[data-item-type="advance"]') ){
					new gdlr_core_advance_settings($(this));
				}
			});

		},
		
		// retrieve all value within the container area
		get_val: function(){
			var obj = {};
			
			this.container.find('[data-slug]').each(function(){
				var slug = $(this).attr('data-slug');
				var input_type = $(this).attr('data-type');
				
				if( input_type == 'text' || input_type == 'textarea' || input_type == 'combobox' ||  
					input_type == 'upload' || input_type == 'colorpicker' || input_type == 'font' ){ 
					obj[slug] = $(this).val();
				}else if( input_type == 'multi-combobox' ){
					if( $(this).val() == null ){
						obj[slug] = "";
					}else{
						obj[slug] = $(this).val().length? $(this).val(): "";
					}
				}else if( input_type == 'checkbox' ){
					obj[slug] = ($(this).is(':checked'))? 'enable': 'disable';
				}else if( input_type == 'radioimage' ){
					if( $(this).is(':checked') ){
						obj[slug] = $(this).val();
					}
				}else if( input_type == 'tinymce' ){
					obj[slug] = gdlr_core_get_tmce_data($(this));
				}else if( input_type == 'custom' ){
					if( $(this).attr('data-item-type') == 'tabs' ){
						$(this).trigger('update_data');
					}

					if( $(this).data('value') ){
						if( $.isArray($(this).data('value')) && $(this).data('value').length == 0 ){
							obj[slug] = '';
						}else{
							obj[slug] = $(this).data('value');
						}
					}else{
						obj[slug] = '';
					}
					
				}
			});
			
			return obj;
			
		}, // get_val
		
		// condition for show/hide item
		init_conditional: function(){	
		
			this.container.find('[data-condition]').each(function(){
				var item = $(this).closest('.gdlr-core-html-option-item, .gdlr-core-html-option-tabs-field');
				var wrapper = item.parent();

				var conditions = JSON.parse($(this).attr('data-condition'));
				var visible = true;

				if( wrapper.is('td, .form-field') ){
					wrapper = wrapper.closest('form');
					for( var key in conditions ){
						
						if( item.is('.gdlr-core-html-option-item') ){
							var obj = wrapper.find('[name="' + key + '"]');
						}

						var obj_val = '';
						if( obj.is('input[type="checkbox"]') ){
							obj_val = (obj.is(':checked'))? 'enable': 'disable';
						}else if( obj.is('input[type="radio"]') ){
							obj_val = obj.filter(':checked').val();
						}else{
							obj_val = obj.val();
						}
						visible = visible && (obj_val == conditions[key] || (conditions[key].constructor === Array && conditions[key].indexOf(obj_val) != -1));
					}
					
					if( !visible ){
						$(this).closest('tr, .form-field').css('display', 'none');
					}else{
						$(this).closest('tr, .form-field').css('display', '');
					}
				}else{
					for( var key in conditions ){
						
						if( item.is('.gdlr-core-html-option-item') ){
							var obj = wrapper.find('[data-slug="' + key + '"], [name="' + key + '"]');
						}else if( item.is('.gdlr-core-html-option-tabs-field') ){
							var obj = wrapper.find('[data-tabs-slug="' + key + '"]');
						}

						var obj_val = '';
						if( obj.is('input[type="checkbox"]') ){
							obj_val = (obj.is(':checked'))? 'enable': 'disable';
						}else if( obj.is('input[type="radio"]') ){
							obj_val = obj.filter(':checked').val();
						}else{
							obj_val = obj.val();
						}
						visible = visible && (obj_val == conditions[key] || (conditions[key].constructor === Array && conditions[key].indexOf(obj_val) != -1));
					}
					
					if( !visible ){
						$(this).hide();
					}else{
						$(this).show();
					}
				}
			});
			
		}, // init_conditional
		bind_conditional: function(){

			this.container.on('change', 'select, input[type="checkbox"], input[type="radio"]', function(){
				var item = $(this).closest('.gdlr-core-html-option-item, .gdlr-core-html-option-tabs-field');
				var wrapper = item.parent();

				if( wrapper.is('td, .form-field') ){
					wrapper = wrapper.closest('form');
					wrapper.find('[data-condition]').each(function(){
						var conditions = JSON.parse($(this).attr('data-condition'));
						var visible = true;
						for( var key in conditions ){
							
							if( item.is('.gdlr-core-html-option-item') ){
								var obj = wrapper.find('[name="' + key + '"]');
							}
							
							var obj_val = '';
							if( obj.is('input[type="checkbox"]') ){
								obj_val = (obj.is(':checked'))? 'enable': 'disable';
							}else if( obj.is('input[type="radio"]') ){
								obj_val = obj.filter(':checked').val();
							}else{
								obj_val = obj.val();
							}
	
							visible = visible && (obj_val == conditions[key] || (conditions[key].constructor === Array && conditions[key].indexOf(obj_val) != -1));
						}
						
						if( !visible ){
							$(this).closest('tr, .form-field').css('display', 'none');
						}else{
							$(this).closest('tr, .form-field').css('display', '');
						}
					});

				}else{
					wrapper.children('[data-condition]').each(function(){
						var conditions = JSON.parse($(this).attr('data-condition'));
						var visible = true;
						for( var key in conditions ){
							
							if( item.is('.gdlr-core-html-option-item') ){
								var obj = wrapper.find('[data-slug="' + key + '"], [name="' + key + '"]');
							}else if( item.is('.gdlr-core-html-option-tabs-field') ){
								var obj = wrapper.find('[data-tabs-slug="' + key + '"]');
							}
							
							var obj_val = '';
							if( obj.is('input[type="checkbox"]') ){
								obj_val = (obj.is(':checked'))? 'enable': 'disable';
							}else if( obj.is('input[type="radio"]') ){
								obj_val = obj.filter(':checked').val();
							}else{
								obj_val = obj.val();
							}
	
							visible = visible && (obj_val == conditions[key] || (conditions[key].constructor === Array && conditions[key].indexOf(obj_val) != -1));
						}
						
						if( !visible ){
							$(this).slideUp(200);
						}else{
							$(this).slideDown(200);
						}
					});
				}
				
			});
			
		}, // bind_conditional
		
		////////////////
		// item event
		////////////////
		
		display_icon_list: function(element){

			var t = this;

			if( t.icon_list_selection == '' ){
				var icon_list_close = $('<div class="gdlr-core-close"></div>');
				icon_list_close.on('click', function(){
					t.icon_list_selection.hide();
					t.icon_list_target = null;
				});

				t.icon_list_selection = $('<div class="gdlr-core-icon-list-selection-lb" ></div>');
				t.icon_list_selection.append(html_option_val.icon_list);
				t.icon_list_selection.append(icon_list_close);
				
				t.icon_list_selection.on('click', '.gdlr-core-html-option-icons-type i', function(){
					if( t.icon_list_target != null ){
						t.icon_list_target.val($(this).attr('class'));
					}
				});

				$('body').append(t.icon_list_selection);
			}

			if( t.icon_list_target != null ){
				t.icon_list_selection.insertAfter(t.icon_list_target);
				t.icon_list_selection.show();
				
				t.icon_list_selection.css({ 
					left: t.icon_list_target.position().left, 
					top: t.icon_list_target.position().top + 50
				})
			}
		},

		bind_input_format: function(){

			var t = this;
			t.container.on('click', 'input[data-input-type="icon"]', function(){
				t.icon_list_target = $(this);
				t.display_icon_list($(this));
			});

			t.container.on('change', 'input[data-input-type]', function(){

				var val = $(this).val();
				var match = val.match(/^-?\d+\.?\d*/g);
				var suffix = '';
				if( $(this).attr('data-input-type') == 'pixel' ){
					if( val.length > 0 && val.charAt(val.length-1) == '%' ){
						suffix = '%';
					}else if( val.length > 0 && val.charAt(val.length-2) == 'e' && val.charAt(val.length-1) == 'm' ){
						suffix = 'em';
					}else{
						suffix = 'px';
					}
				}

				if( typeof(match) != 'undefined' && match != null ){
					$(this).val(match[0] + suffix);
				}

				$(this).trigger('gdlr_core_change');

			});
			t.container.on('keydown', 'input[data-input-type]', function(e){

				var code = (e.keyCode ? e.keyCode : e.which);

				if( code == 40 || code == 38 ){
					var val = $(this).val();
					var match = val.match(/^-?\d+/g);
					var suffix = '';
					if( $(this).attr('data-input-type') == 'pixel' ){
						if( val.length > 0 && val.charAt(val.length-1) == '%' ){
							suffix = '%';
						}else if( val.length > 0 && val.charAt(val.length-2) == 'e' && val.charAt(val.length-1) == 'm' ){
							suffix = 'em';
						}else{
							suffix = 'px';
						}
					}

					if( typeof(match) != 'undefined' && match != null ){
						if( code == 40 ){
							$(this).val( (parseInt(match[0]) - 1) + suffix );
						}else if( code == 38 ){
							$(this).val( (parseInt(match[0]) + 1) + suffix );
						}

						$(this).trigger('gdlr_core_change');
					}
				}

			});

		},

		bind_image_uploader: function(){

			this.container.on('click', '.gdlr-core-upload-image-button', function(){
				
				var image_wrapper = $(this).closest('.gdlr-core-html-option-upload-appearance');
				var image_container = image_wrapper.children('.gdlr-core-upload-image-container');
				var image_input = image_wrapper.children('.gdlr-core-html-option-upload');
				var image_input_url = image_wrapper.children('.gdlr-core-html-option-upload-image');
				
				if( $(this).hasClass('gdlr-core-upload-image-add') ){

					var frame = wp.media({
						title: html_option_val.text.upload_media,
						button: { text: html_option_val.text.choose_media },
						multiple: false
					}).on('select', function(){
			  
						// Get media attachment details from the frame state
						var attachment = frame.state().get('selection').first().toJSON();

						image_wrapper.addClass('gdlr-core-active');
						image_container.css('background-image', 'url(' + attachment.url + ')');
						image_input.val(attachment.id);
						image_input_url.val(attachment.url);

						image_input.trigger('change');
					}).open();
					
				}else if( $(this).hasClass('gdlr-core-upload-image-remove') ){
					image_wrapper.removeClass('gdlr-core-active');
					image_container.css('background-image', '');
					image_input.val('');
					image_input_url.val('');

					image_input.trigger('change');
				}
				
			});
			
		}, // bind image uploader
		
		bind_datepicker: function(){		

			this.container.find('.gdlr-core-html-option-datepicker').each(function(){
				$(this).datepicker({
					dateFormat : 'yy-mm-dd',
					changeMonth: true,
					changeYear: true 
				});
			});

		},
			
		bind_font_slider: function(){
		
			this.container.find('.gdlr-core-html-option-fontslider').each(function(){
				var t = $(this);
				var val = $(this).val();
				var display = $('<div class="gdlr-core-html-option-fontslider-appearance" ></div>');
				var min = ($(this).attr('data-min-value'))? parseInt($(this).attr('data-min-value')): 6;
				var max = ($(this).attr('data-max-value'))? parseInt($(this).attr('data-max-value')): 160;
				var suffix = '';
				if( $(this).attr('data-suffix') != 'none' ){
					if( val.length > 0 && val.charAt(val.length-1) == '%' ){
						suffix = '%';
					}else if( val.length > 0 && val.charAt(val.length-2) == 'e' && val.charAt(val.length-1) == 'm' ){
						suffix = 'em';
					}else{
						suffix = 'px';
					}
				}

				display.insertBefore($(this));
				display.slider({'range': 'min', 'min': min, 'max': max, value: parseInt(val),
					slide: function(event, ui) {
						t.val(ui.value + suffix);
					}
				});
				t.val(parseInt(t.val()) + suffix);

				// update the font slider when input changes
				$(this).on('input change', function(){
					display.slider('value', parseInt(t.val()));
				});
				
			});
		}, // bind font slider

		bind_import: function(){

			this.container.find('.gdlr-core-html-option-import').each(function(){

				var import_form = $(this).find('form');
				var import_file = $(this).find('.gdlr-core-html-option-import-file');
				var import_button = $(this).find('.gdlr-core-html-option-import-button');

				import_button.click(function(){

					if( import_file.val().length > 0 ){
						gdlr_core_confirm_box({ success: function(){
							import_form.submit();
						} });
					}else{
						gdlr_core_alert_box({
							status: 'failed',
							head: 'File not found',
							message: 'Please select the file before importing',
						});
					}

					return false;
				});


			});

		}, // bind import

		bind_export: function(){

			this.container.find('.gdlr-core-html-option-export').each(function(){

				var export_wrap = $(this);
				var export_button = export_wrap.find('.gdlr-core-html-option-export-button');

				export_button.click(function(){

					var data_sent = {
						'action': export_wrap.attr('data-action'),
						'security': html_option_val.text.nonce
					};
					export_wrap.find('.gdlr-core-html-option-export-option').each(function(){
						data_sent.options = $(this).val();
					});

					// ajax to obtain value
					$.ajax({
						type: 'POST',
						url: html_option_val.text.ajaxurl,
						data: data_sent,
						dataType: 'json',
						beforeSend: function(jqXHR, settings){
							export_button.addClass('gdlr-core-now-loading');
						},
						error: function(jqXHR, textStatus, errorThrown){
							gdlr_core_alert_box({ status: 'failed', head: html_option_val.text.error_head, message: html_option_val.text.error_message });
							
							// for displaying the debug text
							console.log(jqXHR, textStatus, errorThrown);
						},
						success: function(data){
							export_button.removeClass('gdlr-core-now-loading');

							if( data.status == 'success' ){
								gdlr_core_download_file(data.url, data.filename);
							}else if( data.status == 'success-2' ){
								gdlr_core_download_content(data.content, data.filename);
							}else if( data.status == 'failed' ){
								gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
							}
						}
					});

				});


			});

		}, // bind export
		
		bind_icon_selector: function(){
			
			// change the icon source
			this.container.on('change', '.gdlr-core-html-option-icons-type-select', function(){
				var selected_option = $(this).val();

				$(this).parent().siblings('.gdlr-core-html-option-icons-type-wrapper')
					.children('[data-icon-type="' + selected_option + '"]').fadeIn(200).addClass('gdlr-core-active')
					.each(function(){ $(this).children('i').css({display: 'inline-block'}); })
					.siblings().css({display: 'none'}).removeClass('gdlr-core-active');
				$(this).parent().siblings('.gdlr-core-html-option-icons-search').val('');
				
				if( selected_option == 'none' ){
					$(this).parent().siblings('.gdlr-core-html-option-icons-search, .gdlr-core-html-option-icons-type-wrapper').slideUp(200);
					$(this).parent().siblings('[data-slug]').val('');
				}else{
					$(this).parent().siblings('.gdlr-core-html-option-icons-search, .gdlr-core-html-option-icons-type-wrapper').slideDown(200);
					$(this).parent().siblings('.gdlr-core-html-option-icons-type-wrapper').slideDown(200)
						.children('.gdlr-core-active').children('.gdlr-core-active').trigger('click');
				}
				
			});
			
			// search for an icon
			this.container.on('input', '.gdlr-core-html-option-icons-search', gdlr_core_debounce(function(){
				var keyword = $(this).val();

				if( keyword ){
					$(this).siblings('.gdlr-core-html-option-icons-type-wrapper').children('.gdlr-core-active')
						.children().css({display: 'inline-block'})
						.not('[class*="' + keyword + '"]').css({display: 'none'});
				}else{
					$(this).siblings('.gdlr-core-html-option-icons-type-wrapper').children('.gdlr-core-active')
						.children().css({display: 'inline-block'});
				}
			}, 500));
			
			// click the icon
			this.container.on('click', '.gdlr-core-html-option-icons-type-wrapper i', function(){
				var icon_container = $(this).closest('.gdlr-core-html-option-icons-type-wrapper');
				
				icon_container.find('i').removeClass('gdlr-core-active');
				icon_container.siblings('[data-slug]').val($(this).attr('class'));
				$(this).addClass('gdlr-core-active');
			});
			
		}, // bind_icon_selector
		
		bind_font_picker: function(){
			
			this.container.on('change', '.gdlr-core-html-option-font', function(){
				
				var display = $(this).parent('.gdlr-core-custom-combobox').siblings('.gdlr-core-html-option-font-display');
				var display_url = display.attr('data-base-url') + '&font-family=' + $(this).val();
				display.attr('src', display_url);
			});
		},
		
		bind_tinymce: function(){
			if( typeof(html_option_val) == 'undefined' ) return;

			this.container.find('.gdlr-core-html-option-tinymce').each(function(){
				gdlr_core_init_tmce_element($(this));
			});
			
		}, // bind_tinymce
		
	}; // gdlrCoreHtmlOption.prototype
	
	//////////////////////////
	// advance settings
	//////////////////////////
	var gdlr_core_advance_settings = function(c){
		this.container = c;

		this.values = c.data('value');
		this.options = c.data('options');

		if( this.options.indexOf('typography') !== -1 ){
			this.options.push('font-size', 'tablet-font-size', 'mobile-font-size', 'font-style', 'font-weight', 'letter-spacing', 'text-transform', 'line-height');
		}

		this.init();
	}

	gdlr_core_advance_settings.prototype = {

		init: function(){
			var t = this;

			if( t.options.indexOf('font-size') !== -1 ){
				t.font_size_options();
			}

			var font_option = $('<div class="gdlr-core-advance-font-option" ></div>');
			if( t.options.indexOf('font-style') !== -1 ){
				font_option.append(t.get_combobox_option('font-style', 'Font Style'));
			}
			if( t.options.indexOf('font-weight') !== -1 ){
				font_option.append(t.get_combobox_option('font-weight', 'Font Weight'));
			}
			if( t.options.indexOf('text-transform') !== -1 ){
				font_option.append(t.get_combobox_option('text-transform', 'Text Transform'));
			}
			if( t.options.indexOf('letter-spacing') !== -1 ){
				font_option.append(t.get_input_option('letter-spacing', 'Letter Spacing'));
			}
			if( t.options.indexOf('line-height') !== -1 ){
				font_option.append(t.get_input_option('line-height', 'Line Height'));
			}
			if( font_option.length ){
				t.container.append(font_option);
			}
			
		},

		font_size_options: function(){
			var t = this;

			var tablet_size = (t.options.indexOf('tablet-font-size') !== -1);
			var mobile_size = (t.options.indexOf('mobile-font-size') !== -1);

			if( tablet_size || mobile_size ){
				var screen_head = $('<div class="gdlr-core-advance-screen-head" ></div>');
				t.container.append(screen_head);

				screen_head.append('<i class="fa fa-desktop" data-advance-tab="desktop" ></i>');
				screen_head.append('<i class="fa fa-tablet" data-advance-tab="tablet" ></i>');
				screen_head.append('<i class="fa fa-mobile" data-advance-tab="mobile" ></i>');

				screen_head.on('click', '[data-advance-tab]', function(){
					var target = $(this).attr('data-advance-tab');
					$(this).parent().next().find('[data-advance-tab="' + target + '"]').each(function(){
						$(this).fadeIn(200);
						$(this).siblings().css('display', 'none');
					});
				});
			}
			
			var screen_content = $('<div class="gdlr-core-advance-screen-content" ></div>');
			t.container.append(screen_content);

			var desktop = $('<div data-advance-tab="desktop" ></div>');
			desktop.append(t.get_input_option('font-size', 'Font Size'));
			screen_content.append(desktop);

			if( tablet_size ){
				var tablet = $('<div data-advance-tab="tablet" ></div>');
				tablet.append(t.get_input_option('tablet-font-size', 'Tablet Font Size'));
				tablet.css('display', 'none');
				screen_content.append(tablet);
			}
			
			if( tablet_size ){
				var mobile = $('<div data-advance-tab="mobile" ></div>');
				mobile.append(t.get_input_option('mobile-font-size', 'Mobile Font Size'));
				mobile.css('display', 'none');
				screen_content.append(mobile);
			}

		},

		get_input_option: function(slug, label){
			var ret = $('<div class="gdlr-core-advance-input-text" ></div>');
			ret.append('<div class="gdlr-core-head" >' + label + '</div>');
			ret.append('<div class="gdlr-core-tail" ><input type="text" data-input-type="pixel" value="" data-advance-slug="' + slug + '" ></div>');

			return ret;
		},

		get_combobox_option: function(slug, label){

			var options;

			switch( slug ){
				case 'font-style':
					options = {
						'normal': 'Normal',
						'italic': 'Italic'
					};
					break;
				case 'font-weight':
					options = {
						'100': '100',
						'200': '200',
						'300': '300',
						'400': '400',
						'500': '500',
						'600': '600',
						'700': '700',
						'800': '800',
						'900': '900'
					};
					break;
				case 'text-transform':
					options = {
						'capitalize': 'Capitalize',
						'uppercase': 'Uppercase',
						'lowercase': 'Lowercase',
						'none': 'None'
					};
					break;

				default:
					options = {};
					break;
			}

			var ret = $('<div class="gdlr-core-advance-combobox" ></div>');
			ret.append('<div class="gdlr-core-head" >' + label + '</div>');
			ret.append('<div class="gdlr-core-tail" ></div>');

			var select = $('<select value="" data-advance-slug="' + slug + '" ></select>');
			select.append('<option value="" ></option>');
			for( var key in options ){
				select.append('<option value="' + key + '" >' + options[key] + '</option>');
			}
			select.wrap('<div class="gdlr-core-tail" ></div>');
			ret.append(select);

			return ret;
		}

	}

	//////////////////////////
	// padding settings
	//////////////////////////
	
	var gdlr_core_padding = function(c){
		
		this.container = c;

		this.values = c.data('value');
		this.options = c.data('options');
		
		this.init();
	}
	gdlr_core_padding.prototype = {
		
		init: function(){
			
			var t = this;
			var input_wrap = $('<div class="gdlr-core-html-option-padding" ></div>');
			var with_link = !t.container.closest('.gdlr-core-html-option-item').hasClass('gdlr-core-no-link');

			// creating an option
			if( typeof(t.options) == 'undefined' || t.options.length == 0 ){
				t.options = ['top', 'right', 'bottom', 'left'];
			}
			for( var key in t.options ){
				var input = $('<input class="gdlr-core-html-option-padding-input" />').attr('data-padding-slug', t.options[key]);
				if( this.container.attr('data-input-type') ){
					input.attr('data-input-type', this.container.attr('data-input-type'));
				}

				if( typeof(t.values) != 'undefined' && typeof(t.values[t.options[key]]) != 'undefined' ){
					input.val(t.values[t.options[key]]);
				}
				
				input_wrap.append( $('<div class="gdlr-core-html-option-padding-item" ></div>')
					.append(input)
					.append('<span>' + t.options[key] + '</span>')
				);
			}
			
			// add link between input box
			if( with_link ){
				var type = 'link';
				if( typeof(t.values) != 'undefined' && typeof(t.values['settings']) != 'undefined' ){ type = t.values['settings']; }
				
				var settings = $('<div class="gdlr-core-html-option-padding-settings" >\
						<i class="fa fa-link" data-padding-type="link" ></i><i class="fa fa-unlink" data-padding-type="unlink" ></i>\
						<input type="hidden" value="' + type + '" data-padding-slug="settings" />\
					</div>');
				settings.find('[data-padding-type="' + type +  '"]').addClass('gdlr-core-active');
				settings.find('[data-padding-type]').click(function(){
					if( $(this).hasClass('gdlr-core-active') ) return;
					
					type = $(this).attr('data-padding-type');
					$(this).addClass('gdlr-core-active').siblings('[data-padding-type]').removeClass('gdlr-core-active');
					$(this).siblings('[data-padding-slug]').val(type);
					
					t.update_data();
				});
				
				input_wrap.append(settings);
			}
			
			// event
			input_wrap.on('input change gdlr_core_change', 'input', function(e){
				if( type == 'link' ){
					$(this).closest('.gdlr-core-html-option-padding-item')
						.siblings('.gdlr-core-html-option-padding-item').find(' [data-padding-slug]').val($(this).val());
				}
				
				t.update_data();
			});
			
			t.container.append(input_wrap);
			
		}, // init
		
		update_data: function(){
			
			var t = this;
			t.values = {};
			
			t.container.find('[data-padding-slug]').each(function(){
				t.values[$(this).attr('data-padding-slug')] = $(this).val();
			});
			
			t.container.data('value', t.values); 
		}		
		
	};
	
	//////////////////////////
	// skin settings
	//////////////////////////
	var gdlr_core_skin_settings = function(c){
		
		this.container = c;
		this.values = c.data('value');

		this.input_name = $('<input class="gdlr-core-html-option-text" placeholder="' + html_option_val.skin.input + '" />');
		this.input_submit = $('<div class="gdlr-core-html-option-skin-input-add" ></div>');
		
		this.skin_container = $('<div class="gdlr-core-html-option-skin-container clearfix" ></div>');
		this.template = $('<div class="gdlr-core-html-option-skin-template" >\
				<div class="gdlr-core-html-option-skin-head" >\
					<input type="hidden" data-skin-slug="name" >\
					<div class="gdlr-core-html-option-skin-name" ></div>\
					<div class="gdlr-core-html-option-skin-remove" ></div>\
				</div>\
			</div>');
		
		// create the template dom
		var template_options = c.data('options');
		var skin_item = $('<div class="gdlr-core-html-option-skin-item" ></div>').appendTo(this.template);
		for( var key in template_options ){
			if( typeof(template_options[key].type) != 'undefined' && template_options[key].type == 'title' ){
				skin_item.append( $('<div class="gdlr-core-html-option-skin-item-section-title" ></div>').append(template_options[key].title) );
			}else{
				skin_item.append( 
					$('<div class="gdlr-core-html-option-skin-item-content"></div>')
						.append( $('<div class="gdlr-core-html-option-skin-item-title" ></div>').append(template_options[key].title) )
						.append( $('<input type="text" class="gdlr-core-html-option-skin-item-color" />').attr({
								'data-skin-slug': key, 
								'data-default': template_options[key].default
							}).val(template_options[key].default) ) );
			}
		}
		
		// init the class
		this.init();
	}
	gdlr_core_skin_settings.prototype = {
		
		init: function(){
			
			var t = this;
			
			// initialte the elements
			$('<div class="gdlr-core-html-option-skin-input-wrapper clearfix" ></div>')
				.append( $('<div class="gdlr-core-html-option-skin-input" >')
					.append(t.input_name)
					.append(t.input_submit) )
				.append( $('<div class="gdlr-core-html-option-item-description" ></div>').html(html_option_val.skin.description) )
				.appendTo(t.container);
			
			t.container.append(t.skin_container);
			
			for( var key in t.values ){
				t.skin_container.append( t.get_template(t.values[key]) );
			}
			
			// bind submit button
			t.bind_submit();
			
			// bind toggle/remove button
			t.bind_head_button();
		},
		
		bind_submit: function(){
			
			var t = this;
			
			t.input_submit.click(function(){
				
				var skin_name = t.input_name.val();
				
				if( skin_name.length > 0 ){
					
					// search if name is duplicated
					for( var key in t.values ){
						if( t.values[key].name == skin_name ){
							gdlr_core_alert_box({
								status: 'failed',
								message: html_option_val.skin.duplicate_input,
								duration: 2500
							});
							return;
						}
					}
					t.input_name.val("");
					t.create_skin(skin_name);
				}else{
					gdlr_core_alert_box({
						status: 'failed',
						message: html_option_val.skin.empty_input,
						duration: 2000
					});
				}
			});
		},
		
		bind_head_button: function(){
			
			var t = this;
			
			t.container.on('click', '.gdlr-core-html-option-skin-remove', function(){
				var skin_remove = $(this);

				gdlr_core_confirm_box({ success: function(){
					skin_remove.closest('.gdlr-core-html-option-skin-template').slideUp(200, function(){
						$(this).remove();
						t.update_data();
					});
				} });

				
				return false;
			});
			
			t.container.on('click', '.gdlr-core-html-option-skin-head', function(){
				$(this).closest('.gdlr-core-html-option-skin-head').siblings('.gdlr-core-html-option-skin-item').slideToggle(200);
			});
		},
		
		create_skin: function( skin_name ){
			
			var t = this;
			var template = t.get_template({name: skin_name});
			
			t.skin_container.append(template);
			template.find('.gdlr-core-html-option-skin-item').css({display: 'block'});
			template.css('display', 'none').slideDown(200, function(){
				t.update_data();
			});
		},
		
		get_template: function( values ){
			
			var t = this;
			var template = t.template.clone();
			
			for( var key in values ){
				if( key == 'name' ){
					template.find('[data-skin-slug="' + key + '"]').val(values[key])
						.siblings('.gdlr-core-html-option-skin-name').html(values[key]);
				}else{
					template.find('[data-skin-slug="' + key + '"]').val(values[key]);
				}
			}
			template.find('.gdlr-core-html-option-skin-item-color').wpColorPicker({
				change: gdlr_core_debounce(function(event, ui){
					t.update_data();
				}, 500),
				clear: function(){
					t.update_data();
				}
			});
			
			return template;
		},
		
		update_data: function(){
			
			var t = this;
			t.values = [];
			
			t.skin_container.find('.gdlr-core-html-option-skin-template').each(function(){
				var skin_item = {};
				$(this).find('[data-skin-slug]').each(function(){
					skin_item[$(this).attr('data-skin-slug')] = $(this).val();
				});
				
				t.values.push(skin_item);
			});
			
			t.container.data('value', t.values); 
		}
		
	}; // gdlr_core_skin_settings.prototype
	
	//////////////////////////
	// gallery settings
	//////////////////////////	
	var gdlr_core_gallery = function(c){
		
		var t = this;
		
		t.container = c;
		t.values = c.data('value');
		t.options = c.data('options');

		t.gallery_container = $('<div class="gdlr-core-html-option-gallery-container" ></div>');
		t.add_button = $('<div class="gdlr-core-html-option-gallery-add" ><i class="icon_plus" ></i></div>');
		
		t.template = $('<div class="gdlr-core-html-option-gallery-template">\
				<img class="gdlr-core-html-option-gallery-template-thumbnail" src="" alt="" /> \
				<div class="gdlr-core-html-option-gallery-template-remove" ><i class="fa fa-remove" ></i></div>\
			</div>');
		
		t.init();
		
	}
	gdlr_core_gallery.prototype = {
		
		init: function(){
			
			var t = this;
			
			// initialize the gallery
			t.container.append(t.add_button).append(t.gallery_container);
			
			for( var key in t.values ){
				t.add_template(t.values[key]);
			}
			
			// bind add image event
			t.bind_add();
			
			// bind option edit
			t.bind_gallery_edit();
			
			// bind remove image event
			t.gallery_container.on('click', '.gdlr-core-html-option-gallery-template-remove', function(){
				$(this).closest('.gdlr-core-html-option-gallery-template').fadeOut(200, function(){
					$(this).remove();
					t.update_data();
				});

				return false;
			});

			t.gallery_container.sortable({
				tolerance: 'pointer',
				stop: function( e, ui ){
					t.update_data();
				}
			});
			
		}, // init
		
		bind_add: function(){
			
			var t = this;
			
			t.add_button.click(function(){
				
				var frame = wp.media({
					title: html_option_val.text.upload_media,
					button: { text: html_option_val.text.choose_media },
					multiple: 'add'
				}).on('select', function(){
		  
					// Get media attachment details from the frame state
					var attachments = frame.state().get('selection').toJSON();
					for( var key in attachments ){
						if( typeof(attachments[key].sizes.thumbnail) != 'undefined' ){
							t.add_template({
								id: attachments[key].id,
								thumbnail: attachments[key].sizes.thumbnail.url
							});
						}else{
							t.add_template({
								id: attachments[key].id,
								thumbnail: attachments[key].sizes.full.url
							});
						}
					}
					t.update_data();
				}).open();
				
			});
			
		}, // bind_add
		
		bind_gallery_edit: function(){
			
			var t = this;

			t.gallery_container.on('click', '.gdlr-core-html-option-gallery-template', function(){
				if( !t.options ) return;
				var gallery_item = $(this);
				var loading = $('<div class="gdlr-core-html-option-gallery-loading" ></div>');
				
				$.ajax({
					type: 'POST',
					url: html_option_val.text.ajaxurl,
					data: { 'action': 'gdlr_get_gallery_options', 'security': html_option_val.text.nonce,
						'options': t.options, 'value': $(this).data('value') },
					dataType: 'json',
					beforeSend: function(jqXHR, settings){
						loading.css({display: 'none'}).appendTo('body');
						loading.fadeIn(150);
					},
					error: function(jqXHR, textStatus, errorThrown){
						gdlr_core_alert_box({ status: 'failed', head: html_option_val.text.error_head, message: html_option_val.text.error_message });
						
						// for displaying the debug text
						console.log(jqXHR, textStatus, errorThrown);
					},
					success: function(data){
						loading.remove();

						if( data.status == 'success' ){
							t.gallery_lb_edit( data.option_content, gallery_item );
						}else if( data.status == 'failed' ){
							gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
						}
					}
				});
			
			});
			
		}, // bind_gallery_edit
		
		gallery_lb_edit: function( content, current_item ){
			
			var t = this;
			var lb_content = $('<div class="gdlr-core-gallery-lightbox-content"></div>');
		
			$('body').append(lb_content);
			lb_content.append(content);
			lb_content.css({opacity: 0}).animate({opacity: 1}, 400);

			// action for html option script
			var html_option = new gdlrCoreHtmlOption(lb_content);
			
			// close lb action
			lb_content.find('#gdlr-core-gallery-lb-head-close').click(function(){
				lb_content.fadeOut(200, function(){
					$(this).remove();
				});
			});

			// save button 
			lb_content.find('#gdlr-core-gallery-lb-options-save').click(function(){
				var new_value = $.extend(current_item.data('value'), html_option.get_val());
				current_item.data('value', new_value);
				t.update_data();
				
				lb_content.fadeOut(200, function(){
					$(this).remove();
				});
			});			
			
		}, // gallery_lb_edit
		
		add_template: function( values ){
			
			var template = this.template.clone();
			
			template.data('value', values);
			template.find('.gdlr-core-html-option-gallery-template-thumbnail').attr('src', values.thumbnail);
			
			this.gallery_container.append(template);

		}, // get_template
		
		update_data: function(){
			var t = this;
			t.values = [];
			
			t.gallery_container.find('.gdlr-core-html-option-gallery-template').each(function(){
				t.values.push($(this).data('value'));
			});
                    
			t.container.data('value', t.values); 
		} // update_data
		
	}; // gdlr_core_gallery.prototype
	
	//////////////////////////
	// key value settings
	//////////////////////////
	var gdlr_core_key_value = function(c){

		var t = this;

		t.container = c;
		t.values = c.data('value');
		t.tab_container = $('<div class="gdlr-core-html-option-key-value-container" >');
		t.add_button = $('<div class="gdlr-core-html-option-key-value-add" ></div>');

		t.item_template = $('<div class="gdlr-core-html-option-key-value-template" >\
				<div class="gdlr-core-html-option-key-value-wrap clearfix" >\
					<div class="gdlr-core-html-option-key-value-first">\
						<input type="text" class="gdlr-core-html-option-key-value-key" data-key-value-slug="key" placeholder="Key" />\
					</div>\
					<div class="gdlr-core-html-option-key-value-second">\
						<input type="text" class="gdlr-core-html-option-key-value-value" data-key-value-slug="value" placeholder="Value" />\
					</div>\
					<div class="gdlr-core-html-option-key-value-remove"></div>\
				</div>\
			</div>');

		t.init();

	}
	gdlr_core_key_value.prototype = {

		init: function(){
			var t = this;
			
			t.container.append(t.tab_container);
			t.container.closest('.gdlr-core-html-option-item-input').siblings('.gdlr-core-html-option-item-title').append(t.add_button);

			t.bind_add();

			// init the content
			if( t.values && t.values.length > 0 ){
				for( var key in t.values ){
					t.tab_container.append(t.get_template(t.values[key]));
				}
			}

			// bind sortable
			t.tab_container.sortable({
				tolerance: 'pointer',
				stop: function( e, ui ){
					t.update_data();
				}
			});

			// bind the remove button
			t.container.on('click', '.gdlr-core-html-option-key-value-remove', function(){
				$(this).closest('.gdlr-core-html-option-key-value-template').slideUp(200, function(){
					$(this).remove();
					
					t.update_data();
				});
			});
			
			// bind the input change
			t.container.on('input change', 'input[type="text"]', gdlr_core_debounce(function(){
				t.update_data();
			}, 500));

		},

		bind_add: function(){
			
			var t = this;
			
			t.add_button.click(function(){
				var template = t.get_template();
				
				t.tab_container.append(template);
				template.css({display: 'none'}).slideDown(200);
				
				t.update_data();
			});
			
		}, // bind_add		

		get_template: function( values ){
			
			var template = this.item_template.clone();
			
			// assign value
			for( var key in values ){

				template.find('[data-key-value-slug="' + key + '"]').each(function(){
					$(this).val(values[key]);
				});

			}			
			
			return template;
			
		}, // get_template		

		update_data: function(){
			var t = this;
			t.values = [];
			
			t.tab_container.find('.gdlr-core-html-option-key-value-template').each(function(){
				var tab_item = {};
				$(this).find('[data-key-value-slug]').each(function(){
					tab_item[$(this).attr('data-key-value-slug')] = $(this).val();
				});
				
				t.values.push(tab_item);
			});

			t.container.data('value', t.values); 

		} // update_data

	} // gdlr_core_key_value.prototype

	//////////////////////////
	// tabs settings
	//////////////////////////
	var gdlr_core_tabs = function(c){
		
		var t = this;
		
		t.container = c;	
		t.values = c.data('value');
		t.extra = c.data('extra');
		
		t.tabs_container = $('<div class="gdlr-core-html-option-tabs-container" ></div>');
		t.add_button = $('<div class="gdlr-core-html-option-tabs-add"></div>');
		
		t.template = {
			text: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<input type="text" class="gdlr-core-html-option-tabs-input gdlr-core-html-option-text" data-tabs-slug="" >\
			</div>',
			textarea: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<textarea class="gdlr-core-html-option-tabs-input gdlr-core-html-option-textarea" data-tabs-slug="" ></textarea>\
			</div>',
			tmce: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<div class="gdlr-core-html-option-tinymce" data-type="tinymce" data-tabs-slug="" ></div>\
			</div>',
			combobox: '<div class="gdlr-core-html-option-tabs-field clearfix" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<div class="gdlr-core-custom-combobox" >\
					<select class="gdlr-core-html-option-combobox" data-type="combobox" data-tabs-slug="" ></select>\
				</div>\
			</div>',
			colorpicker: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<input type="text" class="gdlr-core-html-option-tabs-colorpicker gdlr-core-html-option-colorpicker" data-tabs-slug="" >\
			</div>',
			checkbox: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<label>\
					<input type="checkbox" class="gdlr-core-html-option-checkbox" data-tabs-slug="" checked="checked">\
					<div class="gdlr-core-html-option-checkbox-appearance gdlr-core-noselect">\
						<span class="gdlr-core-checkbox-button gdlr-core-on">' + html_option_val.tabs.tab_checkbox_on + '</span>\
						<span class="gdlr-core-checkbox-separator"></span>\
						<span class="gdlr-core-checkbox-button gdlr-core-off">' + html_option_val.tabs.tab_checkbox_off + '</span>\
					</div>\
				</label>\
			</div>',
			upload: '<div class="gdlr-core-html-option-tabs-field" >\
				<span class="gdlr-core-html-option-tabs-input-title" ></span>\
				<div class="gdlr-core-html-option-upload-appearance" >\
					<input type="hidden" class="gdlr-core-html-option-upload" data-type="upload" data-tabs-slug="" />\
					<input type="hidden" class="gdlr-core-html-option-upload-image" data-type="upload-img" data-tabs-slug="" />\
					<div class="gdlr-core-upload-image-container" style="" ></div>\
					<div class="gdlr-core-upload-image-overlay" >\
						<div class="gdlr-core-upload-image-button-hover">\
							<span class="gdlr-core-upload-image-button gdlr-core-upload-image-add"><i class="icon_plus" ></i></span>\
							<span class="gdlr-core-upload-image-button gdlr-core-upload-image-remove"><i class="icon_minus-06" ></i></span>\
						</div>\
					</div>\
				</div>\
			</div>',
		}

		if( typeof(t.extra) != 'undefined' && typeof(t.extra['item-title']) != 'undefined' ){
			var title_text = t.extra['item-title'];
		}else{
			var title_text = html_option_val.tabs.title_text;
		}
		t.item_template = $('<div class="gdlr-core-html-option-tabs-template" >\
				<div class="gdlr-core-html-option-tabs-template-title" >\
					<span class="gdlr-core-head" >' + title_text + '</span>\
					<div class="gdlr-core-html-option-tabs-remove"></div>\
				</div>\
			</div>');

		var template_options = c.data('options');
		var template_content = $('<div class="gdlr-core-html-option-tabs-template-content" ></div>');
		for( var slug in template_options ){
			var temp = $(t.template[template_options[slug].type]);

			temp.find('[data-tabs-slug]').each(function(){
				if( typeof(template_options[slug]['data-input-type']) != 'undefined' ){
					$(this).attr('data-input-type', template_options[slug]['data-input-type']);
				}

				if( $(this).attr('data-type') == 'upload-img' ){
					$(this).attr('data-tabs-slug', slug + '-img');
				}else{
					$(this).attr('data-tabs-slug', slug);

					// add the option to select box
					if( $(this).attr('data-type') == 'combobox' && typeof(template_options[slug].options) != 'undefined' ){
						for( var option_slug in template_options[slug].options ){
							var combobox_option = $('<option></option>').attr('value', option_slug).html(template_options[slug].options[option_slug]);
							$(this).append(combobox_option);
						}
					}
				}
			});
			
			// default value
			if( template_options[slug].type == 'checkbox' ){
				if( template_options[slug].default == 'disable' ){
					temp.find('input[type="checkbox"]').prop('checked', false);
				}
			}

			// assign title
			temp.find('.gdlr-core-html-option-tabs-input-title').html(template_options[slug].title);

			// assign condition 
			if( typeof(template_options[slug].condition) != 'undefined' ){
				temp.attr('data-condition', JSON.stringify(template_options[slug].condition));
			}

			template_content.append(temp);
		}
		t.item_template.append(template_content);
		
		t.init();
	}
	gdlr_core_tabs.prototype = {
		
		init: function(){
			
			var t = this;
			
			t.container.append(t.tabs_container);
			t.container.closest('.gdlr-core-html-option-item-input').siblings('.gdlr-core-html-option-item-title').append(t.add_button);
			
			// init the content
			if( t.values && t.values.length > 0 ){
				for( var key in t.values ){
					t.tabs_container.append(t.get_template(t.values[key]));
				}

				t.tabs_container.find('.gdlr-core-html-option-colorpicker').wpColorPicker({
					change: gdlr_core_debounce(function(event, ui){
						t.update_data();
					}, 500),
				});

				t.tabs_container.find('.gdlr-core-html-option-tinymce').each(function(){
					gdlr_core_init_tmce_element($(this));
				});

				// trigger the condition
				t.tabs_container.find('select, input[type="checkbox"], input[type="radio"]').trigger('change');
			}

			// bind sortable
			t.tabs_container.sortable({
				tolerance: 'pointer',
				handle: '.gdlr-core-html-option-tabs-template-title',
				stop: function( e, ui ){
					t.update_data();

					var elem;
					if(e.srcElement){
						elem = e.srcElement;
					}else{
						elem = e.toElement;
					}

					// refresh tmce
					$(elem).siblings('.gdlr-core-html-option-tabs-template-content').find('.gdlr-core-html-option-tinymce').each(function(){
						$(this).html(gdlr_core_get_tmce_data($(this)));
						$(this).siblings('.wp-editor-wrap').remove();

						gdlr_core_init_tmce_element($(this));
					});
				}
			});
			
			// bind add button
			t.bind_add();
			
			// bind the remove button
			t.container.on('click', '.gdlr-core-html-option-tabs-remove', function(){
				$(this).closest('.gdlr-core-html-option-tabs-template').slideUp(200, function(){
					$(this).remove();
					
					t.update_data();
				});
			});
				
			// bind the toggle button
			t.container.on('click', '.gdlr-core-html-option-tabs-template-title', function(){
				$(this).siblings('.gdlr-core-html-option-tabs-template-content').slideToggle(200);
			});
			
			// bind the input change
			t.container.on('input change gdlr_core_change', 'input[type="text"], textarea, select', gdlr_core_debounce(function(){
				if( $(this).attr('data-tabs-slug') == 'title' || $(this).attr('data-tabs-slug') == 'sub-title' ){
					var container = $(this).closest('.gdlr-core-html-option-tabs-template-content');
					var tab_title = container.find('[data-tabs-slug="title"]').val();
					var tab_sub_title = container.find('[data-tabs-slug="sub-title"]').val();
					if( typeof(tab_sub_title) == 'undefined' ){
						tab_sub_title = '';
					}
					container.siblings('.gdlr-core-html-option-tabs-template-title').children('.gdlr-core-head').html(tab_title + '<span>' + tab_sub_title + '</span>');
				}
				
				t.update_data();
			}, 500));			
			t.container.on('change', 'input[type="checkbox"], input[type="hidden"], .gdlr-core-html-option-tinymce', function(){
				t.update_data();
			});

			// update event
			t.container.on('update_data', function(){
				t.update_data();
			});
			
		}, // init
		
		bind_add: function(){
			
			var t = this;
			
			t.add_button.click(function(){
				var template = t.get_template();
				
				t.tabs_container.append(template);
				template.find('.gdlr-core-html-option-colorpicker').wpColorPicker({
					change: gdlr_core_debounce(function(event, ui){
						t.update_data();
					}, 500),
				});

				template.find('.gdlr-core-html-option-tinymce').each(function(){
					gdlr_core_init_tmce_element($(this));
				});

				// trigger the condition
				template.find('select, input[type="checkbox"], input[type="radio"]').trigger('change');

				template.find('.gdlr-core-html-option-tabs-template-content').css({display: 'block'});
				template.css({display: 'none'}).slideDown(200);
				
				t.update_data();
			});
			
		}, // bind_add
		
		get_template: function( values ){
			
			var t = this;
			var template = t.item_template.clone();
			
			// assign value
			for( var key in values ){

				template.find('[data-tabs-slug="' + key + '"]').each(function(){
					if( $(this).is('input[type="checkbox"]') ){
						if( values[key] == 'enable' ){
							$(this).prop('checked', true);
						}else{
							$(this).prop('checked', false);
						}
					}else if( $(this).attr('data-type') == 'upload-img' ){
						$(this).val(values[key]);
						$(this).closest('.gdlr-core-html-option-upload-appearance').addClass('gdlr-core-active');
						$(this).siblings('.gdlr-core-upload-image-container').css('background-image', 'url(' + values[key] + ')');
					}else if( $(this).is('.gdlr-core-html-option-tinymce') ){
						$(this).html(values[key]);
					}else{
						$(this).val(values[key]);
					}
				});
			}

			var tab_title = (typeof(values) == 'undefined' || typeof(values['title']) == 'undefined')? '': values['title'];
			var tab_sub_title = (typeof(values) == 'undefined' || typeof(values['sub-title']) == 'undefined')? '': values['sub-title'];	
			if( typeof(tab_sub_title) == 'undefined' ){
				tab_sub_title = '';
			}
			if( tab_title != '' || tab_sub_title != '' ){
				template.find('.gdlr-core-html-option-tabs-template-title').children('.gdlr-core-head').html(tab_title + '<span>' + tab_sub_title + '</span>');
			}

			return template;
			
		}, // get_template
		
		update_data: function(){
			var t = this;
			t.values = [];
			
			t.tabs_container.find('.gdlr-core-html-option-tabs-template').each(function(){
				var tab_item = {};
				$(this).find('[data-tabs-slug]').each(function(){
					if( $(this).is('input[type="checkbox"]') ){
						tab_item[$(this).attr('data-tabs-slug')] = ($(this).is(':checked'))? 'enable': 'disable';
					}else if( $(this).is('.gdlr-core-html-option-tinymce') ){
						tab_item[$(this).attr('data-tabs-slug')] = gdlr_core_get_tmce_data($(this)).replace('&quot;', '"');
					}else{
						tab_item[$(this).attr('data-tabs-slug')] = $(this).val();
					}
				});
				
				t.values.push(tab_item);
			});

			t.container.data('value', t.values); 
		} // update_data
		
	}; // gdlr_core_tabs.prototype
	
	//////////////////////////
	// thumbnail settings
	//////////////////////////
	var gdlr_core_thumbnail_sizing = function(c){
		
		this.container = c;
		this.values = c.data('value');

		this.input_fields = $('<div class="gdlr-core-html-option-thumbnail-sizing-input-wrapper" >\
				<div class="gdlr-core-html-option-thumbnail-sizing-input-name" >\
					<span class="gdlr-core-html-option-thumbnail-sizing-input-title" >' + html_option_val.thumbnail_sizing.name + '</span>\
					<input class="gdlr-core-html-option-thumbnail-sizing-input gdlr-core-html-option-text" data-thumbnail-sizing-slug="name" >\
				</div>\
				<div class="gdlr-core-html-option-thumbnail-sizing-input-options" >\
					<div class="gdlr-core-html-option-thumbnail-sizing-input-width" >\
						<span class="gdlr-core-html-option-thumbnail-sizing-input-title" >' + html_option_val.thumbnail_sizing.width + '</span>\
						<input class="gdlr-core-html-option-thumbnail-sizing-input gdlr-core-html-option-text" data-thumbnail-sizing-slug="width" >\
					</div>\
					<div class="gdlr-core-html-option-thumbnail-sizing-input-height" >\
						<span class="gdlr-core-html-option-thumbnail-sizing-input-title" >' + html_option_val.thumbnail_sizing.height + '</span>\
						<input class="gdlr-core-html-option-thumbnail-sizing-input gdlr-core-html-option-text" data-thumbnail-sizing-slug="height" >\
					</div>\
				</div>\
			</div>');
		this.input_submit = $('<div class="gdlr-core-html-option-thumbnail-sizing-input-add" >' + html_option_val.thumbnail_sizing.add + '</div>').appendTo(this.input_fields);
		this.input_fields.append('<div class="gdlr-core-html-option-thumbnail-sizing-input-description" >' + html_option_val.thumbnail_sizing.description + '</div>');
		
		this.thumbnail_sizing_container = $('<div class="gdlr-core-html-option-thumbnail-sizing-container clearfix" ></div>');
		this.template = $('<div class="gdlr-core-html-option-thumbnail-sizing-template" >\
				<input type="hidden" data-thumbnail-sizing-slug="name" >\
				<input type="hidden" data-thumbnail-sizing-slug="width" >\
				<input type="hidden" data-thumbnail-sizing-slug="height" >\
				<input type="hidden" data-thumbnail-sizing-slug="hard-crop" >\
				<div class="gdlr-core-html-option-thumbnail-sizing-name" ></div>\
				<div class="gdlr-core-html-option-thumbnail-sizing-remove" ></div>\
			</div>');
		
		// init the class
		this.init();
	}
	
	gdlr_core_thumbnail_sizing.prototype = {
		
		init: function(){
			
			var t = this;
			
			t.container.append(t.thumbnail_sizing_container);
			for( var key in t.values ){
				t.thumbnail_sizing_container.append( t.get_template(t.values[key]) );
			}
			
			// initialte the elements
			t.container.append(this.input_fields);
			
			// bind submit button
			t.bind_submit();
			
			// bind remove button
			t.bind_remove_button();
		},
		
		bind_submit: function(){
			
			var t = this;
			
			t.input_submit.click(function(){
				
				var temp_value = {};
				var value_check = true;
				
				t.input_fields.find('[data-thumbnail-sizing-slug]').each(function(){
					if( $(this).val().trim() == '' ){
						value_check = false; return;
					}
					
					if( $(this).attr('data-thumbnail-sizing-slug') == 'width' || $(this).attr('data-thumbnail-sizing-slug') == 'height' ){
						temp_value[$(this).attr('data-thumbnail-sizing-slug')] = parseInt($(this).val());
					}else{
						temp_value[$(this).attr('data-thumbnail-sizing-slug')] = $(this).val();
					}
				});
				
				if( value_check ){
					t.create_option(temp_value);
				}else{
					gdlr_core_alert_box({
						status: 'failed',
						message: html_option_val.thumbnail_sizing.empty_input,
						duration: 2000
					});
				}
			});
		},
		
		bind_remove_button: function(){
			
			var t = this;
			
			t.container.on('click', '.gdlr-core-html-option-thumbnail-sizing-remove', function(){
				var thumbnail_remove = $(this);

				gdlr_core_confirm_box({ success: function(){
					thumbnail_remove.closest('.gdlr-core-html-option-thumbnail-sizing-template').slideUp(200, function(){
						$(this).remove();
						t.update_data();
					});
				}});
				
				return false;
			});
		},
		
		create_option: function( values ){
			
			var t = this;
			var template = t.get_template( values );
			
			t.thumbnail_sizing_container.append(template);
			template.css('display', 'none').slideDown(200, function(){
				t.update_data();
			});
		},
		
		
		get_template: function( values ){
			
			var t = this;
			var template = t.template.clone();
			
			for( var key in values ){
				template.find('[data-thumbnail-sizing-slug="' + key + '"]').val(values[key]);
			}
			template.find('.gdlr-core-html-option-thumbnail-sizing-name').html( values['name'] + 
				'<span class="gdlr-core-tail" >' + values['width'] + '*' + values['height'] + 'px</span>'); 
			
			return template;
		},
		
		update_data: function(){
			
			var t = this;
			t.values = [];
			
			t.thumbnail_sizing_container.find('.gdlr-core-html-option-thumbnail-sizing-template').each(function(){
				var thumbnail_sizing_item = {};
				$(this).find('[data-thumbnail-sizing-slug]').each(function(){
					thumbnail_sizing_item[$(this).attr('data-thumbnail-sizing-slug')] = $(this).val();
				});
				
				t.values.push(thumbnail_sizing_item);
			});
			
			t.container.data('value', t.values); 
		}
		
	}; // gdlr_core_thumbnail_sizing.prototype	
	
	//////////////////////////
	// font upload
	//////////////////////////
	var gdlr_core_font_upload = function(c){
		
		this.container = c;
		this.values = c.data('value');
		
		this.add_button = $('<div class="gdlr-core-html-option-font-upload-add"></div>');
		
		this.font_none = $('<div class="gdlr-core-html-option-font-upload-none" ></div>').html(html_option_val.fontupload.none);
		this.font_container = $('<div class="gdlr-core-html-option-font-upload-container" ></div>');
		this.template = $('<div class="gdlr-core-html-option-font-upload-template" >\
				<div class="gdlr-core-html-option-font-upload-remove" ></div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.font_name + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" placeholder="' + html_option_val.fontupload.font_name_p + '" data-fontupload-slug="name" />\
				</div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.eot + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" data-fontupload-slug="eot" />\
					<div class="gdlr-core-html-option-font-upload-button">' + html_option_val.fontupload.button + '</div>\
				</div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.ttf + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" data-fontupload-slug="ttf" />\
					<div class="gdlr-core-html-option-font-upload-button">' + html_option_val.fontupload.button + '</div>\
				</div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.woff + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" data-fontupload-slug="woff" />\
					<div class="gdlr-core-html-option-font-upload-button">' + html_option_val.fontupload.button + '</div>\
				</div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.font_weight + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" value="400" data-fontupload-slug="font-weight" />\
				</div>\
				<div class="gdlr-core-html-option-font-upload-field" >\
					<div class="gdlr-core-html-option-font-upload-title">' + html_option_val.fontupload.font_style + '</div>\
					<input class="gdlr-core-html-option-font-upload-title gdlr-core-html-option-text" value="normal" data-fontupload-slug="font-style" />\
				</div>\
			</div>');
		
		
		this.init();
	}
	gdlr_core_font_upload.prototype = {
		
		init: function(){
			
			var t = this;
			
			t.container.closest('.gdlr-core-html-option-item-input').siblings('.gdlr-core-html-option-item-title').append(t.add_button);
		
			t.container.append(t.font_container);
			t.font_container.append(t.font_none);
			
			if( t.values && t.values.length > 0 ){
				t.font_none.css('display', 'none');
				
				for( var key in t.values ){
					this.font_container.append(this.get_template(t.values[key]));
				}
			}
			
			// when add new button is clicked
			t.bind_add();
			
			// when the font is removed
			t.bind_remove();
			
			// bind an action when input is changed
			t.font_container.on('change', 'input[data-fontupload-slug]', function(){
				t.update_data();
			});
		},
		
		bind_add: function(){
			
			var t = this;
			
			t.add_button.click(function(){
				var template = t.get_template();
				
				t.font_none.slideUp(200);
				
				t.font_container.append(template);
				template.css('display', 'none').slideDown(200);
				t.update_data();
			});
		},
		
		bind_remove: function(){
			
			var t = this;
			
			t.font_container.on('click', '.gdlr-core-html-option-font-upload-remove', function(){
				if( !t.values || t.values.length <= 1 ){
					t.font_none.slideDown(200);
				}
				
				$(this).closest('.gdlr-core-html-option-font-upload-template').slideUp(200, function(){
					$(this).remove();
					t.update_data();
				});
			});
		},
		
		get_template: function( values ){
			
			var template = this.template.clone();
			
			// assign value
			for( var key in values ){
				template.find('[data-fontupload-slug="' + key + '"]').val(values[key]);
			}			
			
			// bind events for newly created template
			template.find('.gdlr-core-html-option-font-upload-button').click(function(){
				
				var font_input = $(this).siblings('.gdlr-core-html-option-font-upload-title');
				var frame = wp.media({
					title: html_option_val.text.upload_media,
					button: { text: html_option_val.text.choose_media },
					multiple: false
				}).on('select', function(){
		  
					// Get media attachment details from the frame state
					var attachment = frame.state().get('selection').first().toJSON();

					font_input.val(attachment.url).trigger('change');
				}).open();				
			});
			
			return template;
		},
		
		update_data: function(){
			
			var t = this;
			t.values = [];
			
			t.font_container.find('.gdlr-core-html-option-font-upload-template').each(function(){
				var font_item = {};
				$(this).find('[data-fontupload-slug]').each(function(){
					font_item[$(this).attr('data-fontupload-slug')] = $(this).val();
				});
				
				t.values.push(font_item);
			});

			t.container.data('value', t.values); 
		}
	}; // gdlr_core_font_upload.prototype
	
	// for autop
	function gdlr_core_autop( text ) {
		var preserve_linebreaks = false,
			preserve_br = false,
			blocklist = 'table|thead|tfoot|caption|col|colgroup|tbody|tr|td|th|div|dl|dd|dt|ul|ol|li|pre' +
				'|form|map|area|blockquote|address|math|style|p|h[1-6]|hr|fieldset|legend|section' +
				'|article|aside|hgroup|header|footer|nav|figure|figcaption|details|menu|summary';

		// Normalize line breaks.
		text = text.replace( /\r\n|\r/g, '\n' );

		// Remove line breaks from <object>.
		if ( text.indexOf( '<object' ) !== -1 ) {
			text = text.replace( /<object[\s\S]+?<\/object>/g, function( a ) {
				return a.replace( /\n+/g, '' );
			});
		}

		// Remove line breaks from tags.
		text = text.replace( /<[^<>]+>/g, function( a ) {
			return a.replace( /[\n\t ]+/g, ' ' );
		});

		// Preserve line breaks in <pre> and <script> tags.
		if ( text.indexOf( '<pre' ) !== -1 || text.indexOf( '<script' ) !== -1 ) {
			preserve_linebreaks = true;
			text = text.replace( /<(pre|script)[^>]*>[\s\S]*?<\/\1>/g, function( a ) {
				return a.replace( /\n/g, '<wp-line-break>' );
			});
		}

		if ( text.indexOf( '<figcaption' ) !== -1 ) {
			text = text.replace( /\s*(<figcaption[^>]*>)/g, '$1' );
			text = text.replace( /<\/figcaption>\s*/g, '</figcaption>' );
		}

		// Keep <br> tags inside captions.
		if ( text.indexOf( '[caption' ) !== -1 ) {
			preserve_br = true;

			text = text.replace( /\[caption[\s\S]+?\[\/caption\]/g, function( a ) {
				a = a.replace( /<br([^>]*)>/g, '<wp-temp-br$1>' );

				a = a.replace( /<[^<>]+>/g, function( b ) {
					return b.replace( /[\n\t ]+/, ' ' );
				});

				return a.replace( /\s*\n\s*/g, '<wp-temp-br />' );
			});
		}

		text = text + '\n\n';
		text = text.replace( /<br \/>\s*<br \/>/gi, '\n\n' );

		// Pad block tags with two line breaks.
		text = text.replace( new RegExp( '(<(?:' + blocklist + ')(?: [^>]*)?>)', 'gi' ), '\n\n$1' );
		text = text.replace( new RegExp( '(</(?:' + blocklist + ')>)', 'gi' ), '$1\n\n' );
		text = text.replace( /<hr( [^>]*)?>/gi, '<hr$1>\n\n' );

		// Remove white space chars around <option>.
		text = text.replace( /\s*<option/gi, '<option' );
		text = text.replace( /<\/option>\s*/gi, '</option>' );

		// Normalize multiple line breaks and white space chars.
		text = text.replace( /\n\s*\n+/g, '\n\n' );

		// Convert two line breaks to a paragraph.
		text = text.replace( /([\s\S]+?)\n\n/g, '<p>$1</p>\n' );

		// Remove empty paragraphs.
		text = text.replace( /<p>\s*?<\/p>/gi, '');

		// Remove <p> tags that are around block tags.
		text = text.replace( new RegExp( '<p>\\s*(</?(?:' + blocklist + ')(?: [^>]*)?>)\\s*</p>', 'gi' ), '$1' );
		text = text.replace( /<p>(<li.+?)<\/p>/gi, '$1');

		// Fix <p> in blockquotes.
		text = text.replace( /<p>\s*<blockquote([^>]*)>/gi, '<blockquote$1><p>');
		text = text.replace( /<\/blockquote>\s*<\/p>/gi, '</p></blockquote>');

		// Remove <p> tags that are wrapped around block tags.
		text = text.replace( new RegExp( '<p>\\s*(</?(?:' + blocklist + ')(?: [^>]*)?>)', 'gi' ), '$1' );
		text = text.replace( new RegExp( '(</?(?:' + blocklist + ')(?: [^>]*)?>)\\s*</p>', 'gi' ), '$1' );

		text = text.replace( /(<br[^>]*>)\s*\n/gi, '$1' );

		// Add <br> tags.
		text = text.replace( /\s*\n/g, '<br />\n');

		// Remove <br> tags that are around block tags.
		text = text.replace( new RegExp( '(</?(?:' + blocklist + ')[^>]*>)\\s*<br />', 'gi' ), '$1' );
		text = text.replace( /<br \/>(\s*<\/?(?:p|li|div|dl|dd|dt|th|pre|td|ul|ol)>)/gi, '$1' );

		// Remove <p> and <br> around captions.
		text = text.replace( /(?:<p>|<br ?\/?>)*\s*\[caption([^\[]+)\[\/caption\]\s*(?:<\/p>|<br ?\/?>)*/gi, '[caption$1[/caption]' );

		// Make sure there is <p> when there is </p> inside block tags that can contain other blocks.
		text = text.replace( /(<(?:div|th|td|form|fieldset|dd)[^>]*>)(.*?)<\/p>/g, function( a, b, c ) {
			if ( c.match( /<p( [^>]*)?>/ ) ) {
				return a;
			}

			return b + '<p>' + c + '</p>';
		});

		// Restore the line breaks in <pre> and <script> tags.
		if ( preserve_linebreaks ) {
			text = text.replace( /<wp-line-break>/g, '\n' );
		}

		// Restore the <br> tags in captions.
		if ( preserve_br ) {
			text = text.replace( /<wp-temp-br([^>]*)>/g, '<br$1>' );
		}

		return text;
	}

	// for wp_nop 
	function gdlr_core_wp_Nop( html ) {
		var blocklist = 'blockquote|ul|ol|li|dl|dt|dd|table|thead|tbody|tfoot|tr|th|td|h[1-6]|fieldset|figure',
			blocklist1 = blocklist + '|div|p',
			blocklist2 = blocklist + '|pre',
			preserve_linebreaks = false,
			preserve_br = false,
			preserve = [];

		if ( ! html ) {
			return '';
		}

		// Protect script and style tags.
		if ( html.indexOf( '<script' ) !== -1 || html.indexOf( '<style' ) !== -1 ) {
			html = html.replace( /<(script|style)[^>]*>[\s\S]*?<\/\1>/g, function( match ) {
				preserve.push( match );
				return '<wp-preserve>';
			} );
		}

		// Protect pre tags.
		if ( html.indexOf( '<pre' ) !== -1 ) {
			preserve_linebreaks = true;
			html = html.replace( /<pre[^>]*>[\s\S]+?<\/pre>/g, function( a ) {
				a = a.replace( /<br ?\/?>(\r\n|\n)?/g, '<wp-line-break>' );
				a = a.replace( /<\/?p( [^>]*)?>(\r\n|\n)?/g, '<wp-line-break>' );
				return a.replace( /\r?\n/g, '<wp-line-break>' );
			});
		}

		// Remove line breaks but keep <br> tags inside image captions.
		if ( html.indexOf( '[caption' ) !== -1 ) {
			preserve_br = true;
			html = html.replace( /\[caption[\s\S]+?\[\/caption\]/g, function( a ) {
				return a.replace( /<br([^>]*)>/g, '<wp-temp-br$1>' ).replace( /[\r\n\t]+/, '' );
			});
		}

		// Normalize white space characters before and after block tags.
		html = html.replace( new RegExp( '\\s*</(' + blocklist1 + ')>\\s*', 'g' ), '</$1>\n' );
		html = html.replace( new RegExp( '\\s*<((?:' + blocklist1 + ')(?: [^>]*)?)>', 'g' ), '\n<$1>' );

		// Mark </p> if it has any attributes.
		html = html.replace( /(<p [^>]+>.*?)<\/p>/g, '$1</p#>' );

		// Preserve the first <p> inside a <div>.
		html = html.replace( /<div( [^>]*)?>\s*<p>/gi, '<div$1>\n\n' );

		// Remove paragraph tags.
		html = html.replace( /\s*<p>/gi, '' );
		html = html.replace( /\s*<\/p>\s*/gi, '\n\n' );

		// Normalize white space chars and remove multiple line breaks.
		html = html.replace( /\n[\s\u00a0]+\n/g, '\n\n' );

		// Replace <br> tags with line breaks.
		html = html.replace( /(\s*)<br ?\/?>\s*/gi, function( match, space ) {
			if ( space && space.indexOf( '\n' ) !== -1 ) {
				return '\n\n';
			}

			return '\n';
		});

		// Fix line breaks around <div>.
		html = html.replace( /\s*<div/g, '\n<div' );
		html = html.replace( /<\/div>\s*/g, '</div>\n' );

		// Fix line breaks around caption shortcodes.
		html = html.replace( /\s*\[caption([^\[]+)\[\/caption\]\s*/gi, '\n\n[caption$1[/caption]\n\n' );
		html = html.replace( /caption\]\n\n+\[caption/g, 'caption]\n\n[caption' );

		// Pad block elements tags with a line break.
		html = html.replace( new RegExp('\\s*<((?:' + blocklist2 + ')(?: [^>]*)?)\\s*>', 'g' ), '\n<$1>' );
		html = html.replace( new RegExp('\\s*</(' + blocklist2 + ')>\\s*', 'g' ), '</$1>\n' );

		// Indent <li>, <dt> and <dd> tags.
		html = html.replace( /<((li|dt|dd)[^>]*)>/g, ' \t<$1>' );

		// Fix line breaks around <select> and <option>.
		if ( html.indexOf( '<option' ) !== -1 ) {
			html = html.replace( /\s*<option/g, '\n<option' );
			html = html.replace( /\s*<\/select>/g, '\n</select>' );
		}

		// Pad <hr> with two line breaks.
		if ( html.indexOf( '<hr' ) !== -1 ) {
			html = html.replace( /\s*<hr( [^>]*)?>\s*/g, '\n\n<hr$1>\n\n' );
		}

		// Remove line breaks in <object> tags.
		if ( html.indexOf( '<object' ) !== -1 ) {
			html = html.replace( /<object[\s\S]+?<\/object>/g, function( a ) {
				return a.replace( /[\r\n]+/g, '' );
			});
		}

		// Unmark special paragraph closing tags.
		html = html.replace( /<\/p#>/g, '</p>\n' );

		// Pad remaining <p> tags whit a line break.
		html = html.replace( /\s*(<p [^>]+>[\s\S]*?<\/p>)/g, '\n$1' );

		// Trim.
		html = html.replace( /^\s+/, '' );
		html = html.replace( /[\s\u00a0]+$/, '' );

		if ( preserve_linebreaks ) {
			html = html.replace( /<wp-line-break>/g, '\n' );
		}

		if ( preserve_br ) {
			html = html.replace( /<wp-temp-br([^>]*)>/g, '<br$1>' );
		}

		// Restore preserved tags.
		if ( preserve.length ) {
			html = html.replace( /<wp-preserve>/g, function() {
				return preserve.shift();
			} );
		}

		return html;
	}

})(jQuery);	