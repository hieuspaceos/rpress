(function($){
	"use strict";

	///////////////////////////////
	// drag & drop
	///////////////////////////////		

	var gdlrCoreCustomSortable = function( container, settings, template ){

		var t = this;

		t.body = $('body');
		t.container = container;
		t.settings = $.extend({
			dragging_threshold: 200,
			helper: '<div class="gdlr-core-page-builder-helper" ></div>',
			placeholder: {
				main_class: 'gdlr-core-page-builder-placeholder',
				wrapper: 'gdlr-core-page-builder-placeholder-wrapper', 
				column: 'gdlr-core-page-builder-placeholder-column',
				flexbox: 'gdlr-core-page-builder-placeholder-flexbox',
				item: 'gdlr-core-page-builder-placeholder-item',
			},
			item_class: {
				wrapper: '.gdlr-core-page-builder-wrapper', 
				column: '.gdlr-core-page-builder-column',
				flexbox: '.gdlr-core-page-builder-flexbox',
				item: '.gdlr-core-page-builder-item',
			},
			container_class: {
				wrapper: '.gdlr-container-sortable',
				column: '.gdlr-container-sortable, .gdlr-core-wrapper-sortable',
				flexbox: '.gdlr-container-sortable, .gdlr-core-wrapper-sortable',
				item: '.gdlr-container-sortable, .gdlr-core-wrapper-sortable, .gdlr-core-column-sortable, .gdlr-core-flexbox-sortable',
			},

			mousedown: function(e){ return true; },
			startdragging: function(e){ },
			dragging: function(e, current_item, placeholder){ },
			updateposition: function(e){ },
			enddragging: function(e){ },

        }, settings);

		t.template =  $.extend({

			container: null,
			selector: '.gdlr-core-pb-list-draggable',

			item_type: function(current_item){ return 'wrapper'; },
			template_item: function(current_item){ return current_item.clone(); }

		}, template);

		t.is_dragging = false;
		t.current_item = null;
		t.current_item_type = '';
		t.target = null;
		t.target_type = 'append';
		t.helper = null;
		t.placeholder = null;

		t.init();
	}
	gdlrCoreCustomSortable.prototype = {

		init: function(){

			var t = this;

			// set draggable class ( all item_class )
			t.draggable_class = '';
			for( var key in t.settings.item_class ){
				if( t.draggable_class != '' ){
					t.draggable_class += ', ';
				}

				t.draggable_class += t.settings.item_class[key];
			}

			// bind container for dragging
			t.bindContainer();
		},

		// bind container for dragging
		bindContainer: function(){
			var t = this;
			var threshold_time;

			var window_height = $(window).height();
			var mousepos_y = 0;
			$(window).on('resize', function(){ window_height = $(window).height(); });

			// mousedown for item in nav bar
			if( t.template.container != null ){
				t.template.container.on('mousedown', t.template.selector, function(e){
					if( t.current_item != null ) return;
					if( e.which != 1 ) return;
					if( !t.settings.mousedown(e) ) return;

					t.is_dragging = true;
					t.current_item = $(this);
					t.current_item_type = t.template.item_type(t.current_item);

					t.body.addClass('gdlr-core-no-selection');
					threshold_time = (new Date()).getTime();
				});
			}

			// mousedown for item in container
			t.container.on('mousedown', function(e){
				if( t.current_item != null ) return;
				if( e.which != 1 ) return;
				if( !t.settings.mousedown(e) ) return;

				t.is_dragging = true;
				t.current_item = $(e.target).closest(t.draggable_class);
				t.current_item.addClass('gdlr-core-item-dragging');
				t.current_item_type = t.check_item_type(t.current_item);

				t.body.addClass('gdlr-core-no-selection');
				threshold_time = (new Date()).getTime();

				// scroll the screen 
				mousepos_y = e.clientY;
				var screenScrolling = setInterval(function(){
					if( t.is_dragging ){
						if( threshold_time == 0 ){
							if( mousepos_y < window_height * 0.15 ){
								$('html, body').scrollTop($(window).scrollTop() - 10);
							}else if( mousepos_y > window_height * 0.85 ){
								$('html, body').scrollTop($(window).scrollTop() + 10);
							}
						}
					}else{
						clearInterval(screenScrolling);
					}
				}, 15);
			});

			// dragging event
			$(document.body).on('mousemove', function(e){
				if( e.which != 1 ) return;
				mousepos_y = e.clientY;

				if( t.is_dragging ){
					if( threshold_time == 0 ){

						// set dragging cursor
						if( t.current_item_type.length ){
							t.set_helper(e.pageX, e.pageY);
						}
						// get closest container base on current item type
						var eTarget = document.elementFromPoint(e.clientX, e.clientY); // wp6.7 fix
						var container = $(eTarget).closest(t.settings.container_class[t.current_item_type]);

						// container has no children
						var container_child = container.children();
						container_child = container_child.not('.' + t.settings.placeholder.main_class);
						container_child = container_child.not('.gdlr-core-item-dragging');
						if( container_child.length == 0 ){
							if( container.length > 0 ){
								t.target = container;
								t.target_type = 'append';
							}else{
								t.target = null;
							}

						// check children in container
						}else{
							container_child.each(function(){
								var item_offset = $(this).offset();
								var itemW = $(this).outerWidth();
								var itemH = $(this).outerHeight();

								t.target = $(this);

								// check if mouse is inside item
								if( e.pageX >= item_offset.left && e.pageX <= item_offset.left + itemW &&
									e.pageY >= item_offset.top && e.pageY <= item_offset.top + itemH ){

									// if column / flexbox check left - right
									if( t.check_item_type($(this)) == 'column' || t.check_item_type($(this)) == 'flexbox' ){

										if( e.pageX <= item_offset.left + (itemW / 2) ){
											t.target_type = 'before';
										}else{
											t.target_type = 'after';
										}

									// if others check top - bottom
									}else{
										if( e.pageY <= item_offset.top + (itemH / 2) ){
											t.target_type = 'before';
										}else{
											t.target_type = 'after';
										}
									}

									return false;

								// outside item
								}else{

									// if column
									if( t.check_item_type($(this)) == 'column' ){

										// check bottom
										if( e.pageY > item_offset.top + itemH ){
											t.target_type = 'after';

										// left
										}else if( e.pageX <= item_offset.left + (itemW / 2) ){
											t.target_type = 'before';
											return false;

										// right
										}else{
											t.target_type = 'after';
										}


									// if others check top - bottom
									}else{
										if( e.pageY <= item_offset.top + (itemH / 2) ){
											t.target_type = 'before';
											return false;
										}else{
											t.target_type = 'after';
										}
									}

								}

							});

						}
						
						// place the plceholder
						t.set_placeholder();

						// call dragging event
						if( t.placeholder != null && t.current_item != null ){
							t.settings.dragging(e, t.current_item, t.placeholder);
						}

					}else{
						if( threshold_time + t.settings.dragging_threshold < (new Date()).getTime() ){

							threshold_time = 0;

							// hide current item once
							if( !t.current_item.is(t.template.selector) ){
								t.current_item.hide();
							}

							// start dragging event
							t.settings.startdragging(e);
						}
					}
				}
			});
			$('body').on('mouseup', function(e){
				if( e.which != 1 ) return;

				// place the item to target if any
				if( t.target != null && t.current_item != null ){

					// position not change
					if( t.placeholder == null || t.placeholder.prev().is(t.current_item) || t.placeholder.next().is(t.current_item) ){
					
					// position changed
					}else{

						// if it's a template, replace the item with the new one
						if( t.current_item.is(t.template.selector) ){
							t.current_item = t.template.template_item(t.current_item);
						}

						if( t.target_type == 'append' ){
							t.target.append(t.current_item);
						}else if( t.target_type == 'before' ){
							t.current_item.insertBefore(t.target);
						}else if( t.target_type == 'after' ){
							t.current_item.insertAfter(t.target);
						}
						
						t.settings.updateposition();
					}
				}

				// remove target
				if( t.target != null ){
					t.target = null;
				} 

				// remove placeholder
				if( t.placeholder != null ){
					t.placeholder.remove();
					t.placeholder = null;
				} 

				// remove helper
				if( t.helper != null ){
					t.helper.remove();
					t.helper = null;
				} 				

				// clear data
				t.is_dragging = false;
				if( t.current_item != null ){
					t.current_item.removeClass('gdlr-core-item-dragging');
					t.current_item.show();
					if( threshold_time == 0 && !t.current_item.is(t.template.selector) ){
						t.current_item.css({opacity: 0, scale: 0.8}).transition({opacity: 1, scale: 1}, 200, function(){
							$(this).css({opacity: "", transform: ""});
						});
					}
					t.current_item = null;
				}

				t.body.removeClass('gdlr-core-no-selection');

				// end dragging event
				t.settings.enddragging();
			});

		},

		// set placeholder
		set_placeholder: function(){
			var t = this;
			
			if( t.target != null && t.current_item != null ){
				
				if( t.placeholder == null ){
					t.placeholder = $('<div ></div>').addClass(t.settings.placeholder.main_class);
					t.placeholder.addClass(t.settings.placeholder[t.current_item_type]);
					t.body.append(t.placeholder);
				}

				if( t.target_type == 'append' ){
					t.target.append(t.placeholder);
				}else if( t.target_type == 'before' ){
					t.placeholder.insertBefore(t.target);
				}else if( t.target_type == 'after' ){
					t.placeholder.insertAfter(t.target);
				}
			}
		},

		// set helper
		set_helper: function(x, y){
			var t = this; 

			if( t.helper == null ){
				t.helper = $(t.settings.helper);
				t.body.append(t.helper);
			}

			t.helper.css({top: y, left: x});
		},

		// check item type
		check_item_type: function( obj ){
			var t = this; 

			for( var key in t.settings.item_class ){
				if( obj.is(t.settings.item_class[key]) ){
					return key;
				}
			}

			return '';
		}
	}

	///////////////////////////////
	// page builder
	///////////////////////////////		
	
	var gdlrCorePageBuilder = function(){

		this.pb = $('#gdlr-core-page-builder');
		this.pb_val = $('#gdlr-core-page-builder-val');
		this.pb_nonce = $('#page_builder_security').val();
		this.pb_tip = this.pb.siblings('#gdlr-core-page-builder-tip');
		
		this.head_nav = this.pb.find('#gdlr-core-page-builder-head-nav');
		this.head_content = this.pb.find('#gdlr-core-page-builder-head-content');
		this.btn_full_screen = this.head_nav.find('#gdlr-core-page-builder-head-full-screen');

		this.btn_reset = this.pb.find('#gdlr-core-page-builder-head-action-reset');
		this.btn_save_template = this.pb.find('#gdlr-core-page-builder-head-action-save-template');
		this.btn_view_mode = this.pb.find('#gdlr-core-page-builder-head-action-mode-section, #gdlr-core-page-builder-head-action-mode-section-alt');
		this.btn_live_mode = this.pb.find('#gdlr-core-page-builder-head-action-live-mode, #gdlr-core-page-builder-head-action-live-mode-alt');
		
		// for undo / redo action
		this.content_stack = [];
		this.content_stack_active = -1;
		this.btn_undo = this.pb.find('#gdlr-core-page-builder-head-action-undo, #gdlr-core-page-builder-head-action-undo-alt');
		this.btn_redo = this.pb.find('#gdlr-core-page-builder-head-action-redo, #gdlr-core-page-builder-head-action-redo-alt');

		this.body_content = this.pb.children('#gdlr-core-page-builder-body');
		this.body_container = this.body_content.children('#gdlr-core-page-builder-container');
		
		this.init();
	}
	
	gdlrCorePageBuilder.prototype = {
		
		init: function(){
			
			var t = this;
			
			// bind action at the head area
			t.bind_fixed_nav();
			t.bind_head_nav();
			t.bind_head_action();
			t.bind_full_screen();
			
			t.bind_view_mode();
			
			// initiate the element in the head area
			t.bind_wrapper_action();
			t.bind_column_action();
			t.bind_flexbox_action();
			t.bind_item_action();
			
			// convert link to non link 
			var content = t.body_content.find('.gdlr-core-page-builder-item-container-preview');
			content.find('a').on('click', function(){
				return false;
			}); 
			if( typeof(Strip) != 'undefined' ){
				Strip.disable();
			}

			// bind the container area to accept element
			t.bind_container_area(t.body_content);
			t.sortable_column_order();
			
			// update the pagebuilder val before saving the page
			$('#post-preview, #publish, #save-post, .editor-post-publish-button, .editor-post-preview').on('click', function(){
				t.pb_update();
			});
			$('#wpbody-content').on('click', '.editor-header__settings, .edit-post-header__settings, .editor-post-publish-button, .editor-post-preview, .editor-post-save-draft', function(){
				t.pb_update();
				console.log(1);
			});

			// add seo filter
			if( typeof(YoastSEO) != 'undefined' && typeof(YoastSEO.app.registerPlugin) == 'function' ){				
				YoastSEO.app.registerPlugin('gdlrCorePB', {status: 'ready'});
				YoastSEO.app.registerModification('content', $.proxy(t.get_seo_content, t), 'gdlrCorePB', 5);
			}else{
				$(window).on('load', function(){
					if( typeof(YoastSEO) != 'undefined' && typeof(YoastSEO.app.registerPlugin) == 'function' ){				
						YoastSEO.app.registerPlugin('gdlrCorePB', {status: 'ready'});
						YoastSEO.app.registerModification('content', $.proxy(t.get_seo_content, t), 'gdlrCorePB', 5);
					}
				});
			}
			
			t.pb.on('gdlr-core-element-change', function(){
				t.pb.trigger('gdlr-core-element-resize');

				// add seo filter
				if( typeof(YoastSEO) != 'undefined' && typeof(YoastSEO.app.pluginReloaded) == 'function' ){
					YoastSEO.app.pluginReloaded('gdlrCorePB');
				}

				// rankmath seo
				if( 'undefined' !== typeof rankMathGutenberg ){
					rankMathGutenberg.refresh( 'content' )
					return
				}

				if( typeof(RankMathApp) != 'undefined' ){
					RankMathApp.refresh( 'content' );
				}
				
			});


			// tooltip text
			if( t.pb_tip.length > 0 ){
				t.pb.on('gdlr-core-show-tooltip', function(e){

					var pb_tip_wrap = t.pb_tip.siblings('.gdlr-core-page-builder-tip');
				 	t.pb_tip.fadeIn(400);
				 	t.pb_tip.find('.gdlr-core-page-builder-tip-dismiss').on('click', function(){
				 		t.pb_tip.fadeOut(400, function(){
				 			$(this).remove();
				 		});
				 	});

				 	t.pb.unbind('gdlr-core-show-tooltip', e.handleObj.handler, e); 

				 	// update tooltip state
				 	t.ajax_pb_update_option('gdlr-core-page-builder-tip', 1);
				});
			}
		},
		
		get_seo_content: function( content ){
			var t = this;
			
			t.pb.find('.gdlr-core-page-builder-item-container-preview').each(function(){
				content += $(this).children().not('script, style').html();
			});
			
			return content;
		},

		/////////////////
		// save action
		/////////////////		
		
		pb_update: function(){
		
			var t = this;
			var pb_var = [];
				
			t.body_container.children('[data-template]').each(function(){
				pb_var.push(t.pb_update_tempalte_val($(this)));
			});
			
			t.pb_val.val(JSON.stringify(pb_var));

		}, // pb_update
		
		pb_update_tempalte_val: function( current_item ){
			
			var t = this;
			var pb_item = {};
			
			pb_item.template = current_item.attr('data-template');
			pb_item.type = current_item.attr('data-type');
			
			// for column item
			if( current_item.attr('data-column') ){
				pb_item.column = current_item.attr('data-column');
			}

			// for sync template
			if( current_item.attr('data-sync-template') ){
				pb_item['sync-template'] = current_item.attr('data-sync-template');
			}
			
			// get the value
			if( current_item.data('value') ){
				pb_item.value = current_item.data('value');
			}
			
			// get inner item
			if( current_item.attr('data-template') == 'wrapper' ){
				
				var inner_container;
				
				if( current_item.attr('data-type') == 'background' ){
					inner_container = current_item.children('.gdlr-core-page-builder-wrapper-content')
						.children('.gdlr-core-page-builder-wrapper-content-margin')
						.children('.gdlr-core-page-builder-wrapper-content-wrap')
						.children() // container class
						.children('.gdlr-core-page-builder-wrapper-container');		
				}else if( current_item.attr('data-type') == 'sidebar' ){
					inner_container = current_item.children()
						.children('.gdlr-core-page-builder-wrapper-sidebar-container')
						.children('.gdlr-core-page-builder-wrapper-sidebar-content')
						.children();	
				}else if( current_item.attr('data-type') == 'flexbox' ){
					inner_container = current_item.children('.gdlr-core-page-builder-flexbox-content')
						.children('.gdlr-core-page-builder-flexbox-content-margin')
						.children('.gdlr-core-page-builder-flexbox-content-wrap')
						.children();	
				}else{
					inner_container = current_item.children('.gdlr-core-page-builder-column-content')
						.children('.gdlr-core-page-builder-column-content-margin')
						.children('.gdlr-core-page-builder-column-content-wrap')
						.children('.gdlr-core-page-builder-column-container');	
				}

				inner_container.children('[data-template]').each(function(){
					if( !pb_item.items ){
						pb_item.items = [];
					}
					
					pb_item.items.push(t.pb_update_tempalte_val($(this)));
				});
			}

			return pb_item;
			
		}, // pb_update_tempalte_val
		
		update_undo_stack: function( redo ){
			if( !redo ){
				if( this.content_stack_active >= page_builder_var.settings.undo_times ){
					this.content_stack.shift();
				}else{
					this.content_stack_active++;
					this.content_stack.splice(this.content_stack_active, page_builder_var.settings.undo_times);
				}
			}
			this.content_stack[this.content_stack_active] = this.body_container.children().gdlr_core_clone();
		}, // update_undo_stack
		
		update_undo_content: function(){
			this.body_container.html( this.content_stack[this.content_stack_active] );
		}, // update_undo_content
		
		/////////////////
		// view
		/////////////////	
		
		bind_view_mode: function(){
			
			var t = this;
			
			t.btn_view_mode.children('.gdlr-core-page-builder-head-action-mode').on('click', function(){
				var mode = $(this).attr('data-mode');
				if( t.body_content.attr('data-mode') == mode ) return;
				
				t.btn_view_mode.attr('data-mode', mode);
				t.body_content.attr('data-mode', mode);
				t.btn_live_mode.attr('data-mode', mode);
				
				if( mode == 'live' ){
					t.body_content.addClass('gdlr-core-pb-livemode').removeClass('gdlr-core-pb-blockmode');
				}else{
					t.body_content.addClass('gdlr-core-pb-blockmode').removeClass('gdlr-core-pb-livemode');
				}		
				
				t.pb.trigger('gdlr-core-element-resize');

				// save the mode state
				t.ajax_pb_update_option('gdlr-core-page-builder-view-mode', mode);
			});
			
			// live mode button
			t.btn_live_mode.children().on('click', function(){
				if( $(this).hasClass('gdlr-core-active') ) return;
				
				var mode = $(this).attr('data-live-mode');
				t.btn_live_mode.children('[data-live-mode="' + mode + '"]').addClass('gdlr-core-active')
					.siblings().removeClass('gdlr-core-active');
				
				if( mode == 'edit' ){
					t.body_content.removeClass('gdlr-core-live-preview').addClass('gdlr-core-live-edit');
				}else{
					t.body_content.removeClass('gdlr-core-live-edit').addClass('gdlr-core-live-preview');
				}
				
				t.pb.trigger('gdlr-core-element-resize');
			});
			
			// set the live mode container
			var live_mode_container_css = $(t.get_live_mode_container_css()).appendTo('body');
			$(window).on('resize', function(){
				live_mode_container_css.remove();
				live_mode_container_css = $(t.get_live_mode_container_css()).appendTo('body');
			});

		}, // bind_view_mode
		
		get_live_mode_container_css: function(){
			var margin = (this.body_content.width() - this.body_container.width()) / 2;
			var style = '.gdlr-core-pb-livemode .gdlr-core-page-builder-wrapper{margin-left:-' + margin + 'px; margin-right:-' + margin + 'px}';
			return '<style type="text/css">' + style + '</style>';
		}, // get_live_mode_container_css
		
		/////////////////
		// action
		/////////////////
		
		// bind fixed nav bar
		bind_fixed_nav: function(){

			var t = this;
			var pb_head = t.pb.children('.gdlr-core-page-builder-head');
			var pb_head_sub = t.pb.children('.gdlr-core-page-builder-head-sub');
			var active_tab, active_content_tab;

			$(window).on('scroll pb-scroll', function(){
				if( pb_head.hasClass('gdlr-core-fixed') ){
					if( $(window).scrollTop() + 32 < t.pb.offset().top + pb_head_sub.height() + 0 ){

						// clone for animation
						var clone = pb_head.clone();
						clone.insertAfter(pb_head);
						clone.slideUp(200, function(){ $(this).remove(); });
 						
 						// remove float head
						pb_head_sub.css('height', '');

						t.head_nav.children('.gdlr-core-page-builder-head-nav-item.gdlr-core-active').removeClass('gdlr-core-active');
						t.head_content.children('.gdlr-core-active').removeClass('gdlr-core-active').hide();

						active_tab.addClass('gdlr-core-active');
						active_content_tab.addClass('gdlr-core-active').show();

						pb_head.removeClass('gdlr-core-fixed').css({width: '', display: 'block'});
					}else if( $(window).scrollTop() + 0 > t.pb.offset().top + t.pb.outerHeight() ){
						pb_head.addClass('gdlr-core-outside').slideUp(200);
					}else if( pb_head.hasClass('gdlr-core-outside') ){
						pb_head.removeClass('gdlr-core-outside').slideDown(200);
					}

				}else{
					if( $(window).scrollTop() + 32 >= t.pb.offset().top + pb_head.outerHeight() + 0 && 
						$(window).scrollTop() + 0 < t.pb.offset().top + t.pb.outerHeight() ){

						pb_head_sub.css('height', pb_head.outerHeight());
						
						active_tab = t.head_nav.children('.gdlr-core-page-builder-head-nav-item.gdlr-core-active').removeClass('gdlr-core-active');
						active_content_tab = t.head_content.children('.gdlr-core-active').removeClass('gdlr-core-active').hide();

						pb_head.addClass('gdlr-core-fixed').css({display: 'none', width: t.pb.outerWidth()}).slideDown(200);
					}
				}
			});
			$(window).on('resize', function(){
				if( pb_head.hasClass('gdlr-core-fixed') ){
					pb_head.css({width: t.pb.outerWidth()});
				}
			});

			// full screen button click
			t.btn_full_screen.on('click', function(){
				if( pb_head.hasClass('gdlr-core-fixed') ){
					pb_head.hide();
				}
			});
		},

		// bind action at the head area
		bind_head_nav: function(){
			
			var t = this;
			
			// nav change event
			t.head_nav.children('.gdlr-core-page-builder-head-nav-item').on('click', function(){

				var nav_slug = $(this).attr('data-nav-type');

				if( $(this).hasClass('gdlr-core-active') ){ 
					$(this).removeClass('gdlr-core-active');
					t.head_content.children('[data-content-type="' + nav_slug + '"]')
						.removeClass('gdlr-core-active').css('display', 'block').slideUp(150);
				}else{
					$(this).addClass('gdlr-core-active');
					$(this).siblings('.gdlr-core-page-builder-head-nav-item').removeClass('gdlr-core-active');

					if( t.head_content.children('.gdlr-core-active').length ){
						t.head_content.children('.gdlr-core-active').removeClass('gdlr-core-active').hide();
						t.head_content.children('[data-content-type="' + nav_slug + '"]').addClass('gdlr-core-active').css({display: 'none'}).fadeIn(150);
					}else{
						t.head_content.children('[data-content-type="' + nav_slug + '"]').addClass('gdlr-core-active').css({display: 'none'}).slideDown(150);
					}
				}
			});
			
			// search nav 
			t.head_content.on('input', '.gdlr-core-page-builder-head-content-search', gdlr_core_debounce(function(){
				var search_val = $(this).val();
				
				if( search_val.length > 0 ){
					$(this).siblings().children('.gdlr-core-pb-list-draggable').each(function(){
						if( $(this).is('[data-title*="' + search_val + '" i], [data-type*="' + search_val + '"]') ){
							$(this).css({display: 'block'});
						}else{
							$(this).css({display: 'none'});
						}
					});
				}else{
					$(this).siblings().children('.gdlr-core-pb-list-draggable').css({display: 'block'});
				}
				
			}, 300));
			
			// template button
			t.head_content.find('.gdlr-core-template-type-button').on('click', function(){
				if( $(this).hasClass('gdlr-core-active') ) return; 
				
				$(this).addClass('gdlr-core-active').siblings().removeClass('gdlr-core-active');
				
				var tab_click = $(this).attr('data-type');
				$(this).parent().siblings('.gdlr-core-page-builder-head-content-template-container').each(function(){
					if( $(this).attr('data-type') == tab_click ){
						$(this).addClass('gdlr-core-active').css({display: 'none'}).fadeIn(150);
					}else{
						$(this).removeClass('gdlr-core-active').hide();
					}
				});
			});
		
			// nav item click
			t.bind_head_nav_click();
			
			// custom template remove
			t.head_content.on('click', '.gdlr-core-page-builder-head-content-custom-template-remove', function(){
		
				var custom_template = $(this).closest('.gdlr-core-page-builder-head-content-custom-template-item'); 
		
				gdlr_core_confirm_box({ success: function(){
					t.ajax_get_template( null, { 
						data: { 'security': t.pb_nonce, 'action': 'gdlr_core_remove_pb_custom_template', 'slug': custom_template.attr('data-template-slug') },
						success: function( data ){ 
							if( data.status == 'failed' ){
								template.slideUp(200, function(){ $(this).remove() });
								gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
							}
						},
					});
					
					custom_template.fadeOut(200, function(){ $(this).remove(); });
				}});
				
				return false;
			});

			// custom template remove
			t.head_content.on('click', '.gdlr-core-page-builder-head-content-sync-template-remove', function(){
		
				var custom_template = $(this).closest('.gdlr-core-page-builder-head-content-sync-template-item'); 
		
				gdlr_core_confirm_box({ 
					text: page_builder_var.sync_template.sync_template_remove_head,
					sub: page_builder_var.sync_template.sync_template_remove_message,
					success: function(){
						t.ajax_get_template( null, { 
							data: { 'security': t.pb_nonce, 'action': 'gdlr_core_remove_pb_sync_template', 'slug': custom_template.attr('data-template-slug') },
							success: function( data ){ 
								if( data.status == 'failed' ){
									template.slideUp(200, function(){ $(this).remove() });
									gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
								}
							},
						});
						
						custom_template.fadeOut(200, function(){ $(this).remove(); });
					}
				});
				
				return false;
			});
			
		}, // bind_head_nav
		
		bind_head_nav_click: function(){
			
			var t = this;
			

			t.head_content.on('click', '.gdlr-core-pb-list-draggable', function(e){
				if( $(e.target).is('.fa-remove') ) return;

				var eTarget = document.elementFromPoint(e.clientX, e.clientY); // fix wp6.7
				if( !$(eTarget).closest(e.target).length ) return; // fix wp6.7
				
				var template = t.get_template($(this));
				
				// add new template to container area
				t.body_container.append(template);
				t.update_undo_stack();
				template.css({opacity: 0, scale: 0.8}).transition({opacity: 1, scale: 1}, 200, function(){
					$(this).css({opacity: "", transform: ""});
				});
				
				// alert message
				gdlr_core_alert_box({ status: 'success', head: page_builder_var.text.item_added, duration: 1000});

				// show tooltop if any
				t.pb.trigger('gdlr-core-show-tooltip');
				
				// reorder the column again
				t.sortable_column_order();

			});
			
		}, // bind_head_nav_item
		
		bind_full_screen: function(){
			
			var t = this;
			var wpwrap = $('#wpwrap');
			var wpadminbar = $('#wpadminbar');
			var pb_parent = t.pb.parent();
			

			t.btn_full_screen.on('click', function(){
				
				// click full screen button
				if( !$(this).hasClass('gdlr-core-active') ){

					$(this).addClass('gdlr-core-active');

					// remove wordpress settings
					wpwrap.css({ display: 'none' });
					$('html').css({ 'padding-top': 0 });

					// allocate new position
					$('body').append(t.pb.addClass('gdlr-core-fullscreen'));
					$('html, body').scrollTop(0).trigger('resize').trigger('scroll');
					
				// revert full screen button
				}else{

					$(this).removeClass('gdlr-core-active');

					// return wordpress settings
					wpwrap.css({ display: '' });
					$('html').css({ 'padding-top': '' });

					// return original position
					pb_parent.append(t.pb);
					t.pb.removeClass('gdlr-core-fullscreen');

					// scroll position
					$(window).scrollTop(t.pb.offset().top).trigger('resize').trigger('scroll');
				}
			});

			// ajax save page builder
			t.head_nav.find('.gdlr-core-page-builder-head-nav-update').on('click', function(){
				if( $(this).hasClass('gdlr-core-now-loading') ) return; 

				var nav_update_button = $(this).addClass('gdlr-core-now-loading');
				var post_id = $(this).attr('data-post-id');
				var value = [];
				t.body_container.children().each(function(){
					value.push(t.pb_update_tempalte_val($(this)));
				});

				var ajax_data = { 
					'security': t.pb_nonce, 
					'action': 'gdlr_core_save_page_builder_data', 
					'post_id': post_id, 
					'value': JSON.stringify(value) 
				};
				
				// check if custom template
				if( !post_id ){
					$(this).closest('.gdlr-core-pb-template-settings').each(function(){
						$(this).find('#gdlr-core-pb-template-settings-head [data-template-slug]').each(function(){
							ajax_data[$(this).attr('data-template-slug')] = $(this).val();
						});
					});
				}
				console.log(ajax_data);
				
				$.ajax({
					type: 'POST',
					url: page_builder_var.text.ajaxurl,
					data: ajax_data,
					dataType: 'json',
					error: function(jqXHR, textStatus, errorThrown){
						nav_update_button.removeClass('gdlr-core-now-loading');
						gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
						
						// for displaying the debug text
						console.log(jqXHR, textStatus, errorThrown);
					},
					success: function(data){
						nav_update_button.removeClass('gdlr-core-now-loading');
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});	
					}
				});	

			});
			
		}, // bind_full_screen		
		
		// template for wrapper / column / item
		get_template: function( elem ){
			
			var t = this;
			var template = '';
			
			if(	elem.attr('data-template') == 'wrapper' ){ 	
				
				if( elem.attr('data-type') == 'column' ){ 
				
					var column_num = elem.attr('data-column');
					template = $(page_builder_var.template.column);

					template.addClass('gdlr-core-column-' + column_num).attr('data-column', column_num);
					template.find('.gdlr-core-page-builder-column-head-title').html(page_builder_var.column[column_num]);
				
				}else{
					template = $(page_builder_var.template[elem.attr('data-type')]);
				}

				t.update_element_preview(template);

			}else if( elem.attr('data-template') == 'element' ){

				template = $(page_builder_var.template.element);
				template.attr('data-type', elem.attr('data-type'));
				template.find('.gdlr-core-page-builder-item-container-item-title, .gdlr-core-page-builder-item-head-item-title').html( elem.attr('data-title') );
				
				t.update_element_preview(template);

			}else if( elem.attr('data-template') == 'template' ){
				
				template = $(page_builder_var.template.template);
				template.attr('data-type', elem.attr('data-type'));
				template.attr('data-template-slug', elem.attr('data-template-slug'));
				
				t.ajax_get_template( template, { 
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_template', 'slug': elem.attr('data-template-slug') }
				});

			}else if( elem.attr('data-template') == 'custom-template' ){
				
				template = $(page_builder_var.template.template);
				template.attr('data-type', elem.attr('data-type'));
				template.attr('data-template-slug', elem.attr('data-template-slug'));
				
				t.ajax_get_template( template, { 
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_custom_template', 'slug': elem.attr('data-template-slug') }
				});

			}else if( elem.attr('data-template') == 'sync-template' ){
				
				template = $(page_builder_var.template.template);
				template.attr('data-type', elem.attr('data-type'));
				template.attr('data-template-slug', elem.attr('data-template-slug'));
				
				t.ajax_get_template( template, { 
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_sync_template', 'slug': elem.attr('data-template-slug') }
				});

			}
			
			return template;
			
		}, // get_template
		
		// bind wrapper action button
		bind_wrapper_action: function(){
			
			var t = this;
			
			// remove wrapper item
			t.body_content.on('click', '.gdlr-core-page-builder-wrapper .gdlr-core-page-builder-wrapper-remove', function(){
				$(this).closest('.gdlr-core-page-builder-wrapper').fadeOut(200, function(){
					$(this).remove();
					
					t.pb.trigger('gdlr-core-element-change');
					t.update_undo_stack();
				});
			});
			
			// edit button
			t.body_content.on('dblclick', '.gdlr-core-page-builder-wrapper', function(){
				var wrapper = $(this);
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': wrapper.attr('data-template'), 'type': wrapper.attr('data-type'), 
							'value': wrapper.data('value') 
					}
				}, wrapper);
			
				return false;
			});		
			t.body_content.on('click', '.gdlr-core-page-builder-wrapper .gdlr-core-page-builder-wrapper-edit', function(){
				var wrapper = $(this).closest('.gdlr-core-page-builder-wrapper');
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': wrapper.attr('data-template'), 'type': wrapper.attr('data-type'), 
							'value': wrapper.data('value') 
					}
				}, wrapper);
			});		

			// save template
			t.body_content.on('click', '.gdlr-core-page-builder-wrapper .gdlr-core-page-builder-wrapper-save', function(){
				var value = [];
				value.push(t.pb_update_tempalte_val($(this).closest('.gdlr-core-page-builder-wrapper')));
				t.lb_save_custom_template(value, 'wrapper');
			});
			
			// sync template
			t.body_content.on('click', '.gdlr-core-page-builder-wrapper .gdlr-core-page-builder-wrapper-sync', function(){
				var value = [];
				var current_item = $(this).closest('.gdlr-core-page-builder-wrapper');
				value.push(t.pb_update_tempalte_val(current_item));
				t.lb_save_sync_template(value, 'wrapper', current_item);
			});
			
			// copy
			t.body_content.on('click', '.gdlr-core-page-builder-wrapper .gdlr-core-page-builder-wrapper-copy', function(){
				var wrapper = $(this).closest('.gdlr-core-page-builder-wrapper');
				var clone = wrapper.gdlr_core_clone();
				
				clone.insertAfter(wrapper);
				clone.css({opacity: 0, scale: 0}).transition({opacity: 1, scale: 1}, 150, function(){
					$(this).css({opacity: "", transform: ""});
				});
				
				t.update_undo_stack();
			});	
			
		}, // bind_wrapper_action
		
		// action button for column item
		bind_column_action: function(){
			
			var t = this;
			
			// increase - decrease item size
			t.body_container.on('click', '.gdlr-core-page-builder-column .gdlr-core-page-builder-column-size i', function(){
				var column_item = $(this).closest('.gdlr-core-page-builder-column');
				var column_num = column_item.attr('data-column');
				var column_text = $(this).closest('.gdlr-core-page-builder-column-head').find('.gdlr-core-page-builder-column-head-title');
				
				var prev = 10, next = 60;
				for( var key in page_builder_var.column ){
					if( key < column_num ){
						prev = key;
					}else if( key > column_num ){
						next = key;  break;
					}
				}
				
				if( $(this).hasClass('gdlr-core-page-builder-column-increase') ){
					if( column_num != next ){
						column_item.removeClass('gdlr-core-column-' + column_num);
						
						column_text.html(page_builder_var.column[next]);
						column_item.addClass('gdlr-core-column-' + next).attr('data-column', next);
					}
					
					// reorder the column again
					t.sortable_column_order();
					
					t.pb.trigger('gdlr-core-element-resize');
					t.update_undo_stack();
					
				}else if( $(this).hasClass('gdlr-core-page-builder-column-decrease') ){
					if( column_num != prev ){
						column_item.removeClass('gdlr-core-column-' + column_num);
						
						column_text.html(page_builder_var.column[prev]);
						column_item.addClass('gdlr-core-column-' + prev).attr('data-column', prev);
					}
					
					// reorder the column again
					t.sortable_column_order();
					
					t.pb.trigger('gdlr-core-element-resize');
					t.update_undo_stack();
				}
			});
			
			// remove column item
			t.body_content.on('click', '.gdlr-core-page-builder-column .gdlr-core-page-builder-column-remove', function(){
				$(this).closest('.gdlr-core-page-builder-column').fadeOut(200, function(){
					$(this).remove();
					
					t.sortable_column_order();
					
					t.pb.trigger('gdlr-core-element-change');
					t.update_undo_stack();
				});
			});
			
			// edit button
			t.body_content.on('dblclick', '.gdlr-core-page-builder-column', function(e){

				// if click on increase/decrease column size
				if( $(e.target).is('.gdlr-core-page-builder-column-increase, .gdlr-core-page-builder-column-decrease') ){
					return false;
				}

				var column = $(this);
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': column.attr('data-template'), 'type': column.attr('data-type'),
							'value': column.data('value') 
					}
				}, column);

				return false;
			});	
			t.body_content.on('click', '.gdlr-core-page-builder-column .gdlr-core-page-builder-column-edit', function(){
				var column = $(this).closest('.gdlr-core-page-builder-column');
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': column.attr('data-template'), 'type': column.attr('data-type'),
							'value': column.data('value') 
					}
				}, column);
			});			
			
			// save template
			t.body_content.on('click', '.gdlr-core-page-builder-column .gdlr-core-page-builder-column-save', function(){
				var value = [];
				value.push(t.pb_update_tempalte_val($(this).closest('.gdlr-core-page-builder-column')));
				t.lb_save_custom_template(value, 'column');
			});
			
			// copy
			t.body_content.on('click', '.gdlr-core-page-builder-column .gdlr-core-page-builder-column-copy', function(){
				var column = $(this).closest('.gdlr-core-page-builder-column');
				var clone = column.gdlr_core_clone();

				clone.insertAfter(column);
				clone.css({opacity: 0, scale: 0}).transition({opacity: 1, scale: 1}, 150, function(){
					$(this).css({opacity: "", transform: ""});
				});
				
				// reorder the column again
				t.sortable_column_order();
				
				t.update_undo_stack();
				
			});			
			
			
		}, // bind_column_action

		bind_flexbox_action: function(){

			var t = this;

			// remove flexbox item
			t.body_content.on('click', '.gdlr-core-page-builder-flexbox .gdlr-core-page-builder-flexbox-remove', function(){
				$(this).closest('.gdlr-core-page-builder-flexbox').fadeOut(200, function(){
					$(this).remove();
					
					t.pb.trigger('gdlr-core-element-change');
					t.update_undo_stack();
				});
			});
			
			// edit button
			t.body_content.on('dblclick', '.gdlr-core-page-builder-flexbox', function(e){
				var column = $(this);
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': column.attr('data-template'), 'type': column.attr('data-type'),
							'value': column.data('value') 
					}
				}, column);

				return false;
			});	
			t.body_content.on('click', '.gdlr-core-page-builder-flexbox .gdlr-core-page-builder-flexbox-edit', function(){
				var column = $(this).closest('.gdlr-core-page-builder-flexbox');
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': column.attr('data-template'), 'type': column.attr('data-type'),
							'value': column.data('value') 
					}
				}, column);
			});	
			
			// copy
			t.body_content.on('click', '.gdlr-core-page-builder-flexbox .gdlr-core-page-builder-flexbox-copy', function(){
				var flexbox = $(this).closest('.gdlr-core-page-builder-flexbox');
				var clone = flexbox.gdlr_core_clone();

				clone.insertAfter(flexbox);
				clone.css({opacity: 0, scale: 0}).transition({opacity: 1, scale: 1}, 150, function(){
					$(this).css({opacity: "", transform: ""});
				});
				
				t.update_undo_stack();
				
			});			

		},
		
		// action button for item's item
		bind_item_action: function(){
			
			var t = this;
			
			// remove item
			t.body_content.on('click', '.gdlr-core-page-builder-item .gdlr-core-page-builder-item-remove', function(){
				$(this).closest('.gdlr-core-page-builder-item').fadeOut(200, function(){
					$(this).remove();
					
					t.pb.trigger('gdlr-core-element-change');
					t.update_undo_stack();
				});
			});
			
			// edit button
			t.body_content.on('dblclick', '.gdlr-core-page-builder-item', function(){
				var item = $(this);
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': item.attr('data-template'), 'type': item.attr('data-type'), 
							'value': item.data('value') 
					}
				}, item);
				
				return false;
			});
			t.body_content.on('click', '.gdlr-core-page-builder-item .gdlr-core-page-builder-item-edit', function(){
				var item = $(this).closest('.gdlr-core-page-builder-item');
				
				t.ajax_get_options({
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_options',
							'template': item.attr('data-template'), 'type': item.attr('data-type'), 
							'value': item.data('value') 
					}
				}, item);
			});
			
			// save template
			t.body_content.on('click', '.gdlr-core-page-builder-item .gdlr-core-page-builder-item-save', function(){
				var value = [];
				value.push(t.pb_update_tempalte_val($(this).closest('.gdlr-core-page-builder-item')));
				t.lb_save_custom_template(value, 'element');
			});
			
			// copy
			t.body_content.on('click', '.gdlr-core-page-builder-item .gdlr-core-page-builder-item-copy', function(){
				var item = $(this).closest('.gdlr-core-page-builder-item');
				var clone = item.gdlr_core_clone();
				
				clone.insertAfter(item);
				clone.css({opacity: 0, scale: 0}).transition({opacity: 1, scale: 1}, 150, function(){
					$(this).css({opacity: "", transform: ""});
				});
				t.update_undo_stack();
			});
			
		}, // bind_item_action	
		
		bind_head_action: function(){
			
			var t = this;
			
			// undo/redo action
			t.update_undo_stack();
			t.btn_undo.on('click', function(){
				if( t.content_stack_active > 0 ){
					t.content_stack_active--;
					t.update_undo_content();
				}else{
					gdlr_core_alert_box({status: 'success', head: page_builder_var.text.undo_end})
				}
			});
			t.btn_redo.on('click', function(){
				if( typeof(t.content_stack[t.content_stack_active + 1]) != 'undefined' ){
					t.content_stack_active++;
					t.update_undo_content();
				}else{
					gdlr_core_alert_box({status: 'success', head: page_builder_var.text.redo_end})
				}
			});
			
			// reset button
			t.btn_reset.on('click', function(){
				gdlr_core_confirm_box({
					success: function(){
						t.body_container.empty();
						t.update_undo_stack();
					},
					sub: ''
				});
			});
			
			// save template button
			t.btn_save_template.on('click', function(){
				var value = [];
				t.body_container.children().each(function(){
					value.push(t.pb_update_tempalte_val($(this)));
				});
				console.log(window.gdlr_core_array_count(value));

				t.lb_save_custom_template(value, 'wrapper');				
			});
			t.btn_save_template.on('contextmenu', function(){
				var value = [];
				t.body_container.children().each(function(){
					value.push(t.pb_update_tempalte_val($(this)));
				});
				
				console.log(JSON.stringify(value).replace(/\'/g, "\\\'"));
			});
			
		}, // bind_head_action
		
		///////////////////
		// ajax action
		///////////////////
		
		// update option table
		ajax_pb_update_option: function( slug, value ){

			var t = this;

			if( typeof(slug) != 'undefined' && typeof(value) != 'undefined' ){
				$.ajax({
					type: 'POST',
					url: page_builder_var.text.ajaxurl,
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_pb_update_option', 'slug': slug, 'value': value },
					dataType: 'json',
					error: function(jqXHR, textStatus, errorThrown){
						// for displaying the debug text
						console.log(jqXHR, textStatus, errorThrown);
					},
					success: function(data){
						if( typeof(data) != 'undefined' && data ){
							console.log(data);
						}
					}
				});	
			}
		},

		// get page builder item options
		ajax_get_options: function( options, current_item ){
			
			var t = this;
			var loading = $('<div class="gdlr-core-page-builder-loading" ></div>');
			var settings = $.extend({
				beforeSend: function(jqXHR, settings){
					var left_pos = t.pb.offset().left + (t.pb.outerWidth() / 2);

					loading.css({display: 'none', 'left': left_pos}).appendTo('body');
					loading.fadeIn(150);
				},
				error: function(jqXHR, textStatus, errorThrown){
					loading.remove();

					gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
					
					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					loading.remove();
					
					if( data.status == 'success' ){
						t.lb_options(data.option_content, current_item);
					}else if( data.status == 'failed' ){
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
					}
				}
			}, options);
			
			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: settings.data,
				dataType: 'json',
				beforeSend: settings.beforeSend,
				error: settings.error,
				success: settings.success
			});	
			
		},		

		// get skin options
		ajax_get_skin_options: function( combobox ){
			
			var t = this;
			var loading = $('<div class="gdlr-core-page-builder-loading" ></div>');

			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_skin_options' },
				dataType: 'json',
				beforeSend: function(jqXHR, settings){
					loading.css({display: 'none'}).appendTo('body');
					loading.fadeIn(150);
				},
				error: function(jqXHR, textStatus, errorThrown){
					loading.remove();

					gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
					
					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					loading.remove();

					if( data.status == 'success' ){
						t.lb_skin_options( data.option_content, combobox );
					}else if( data.status == 'failed' ){
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
					}
				}
			});	
			
		},

		// ajax save skin
		ajax_update_skin: function( value, combobox ){
			
			var t = this;

			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: { 'security': t.pb_nonce, 'action': 'gdlr_core_update_skin', 'value': value, 'skin': combobox.val() },
				dataType: 'json',
				error: function(jqXHR, textStatus, errorThrown){
					gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
					
					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					console.log(data);
					if( data.status == 'success' ){
						console.log(data);
						if( typeof(data.combobox) != 'undefined' ){
							combobox.html(data.combobox);
						}
						if( typeof(data.style) != 'undefined' ){
							var temp_style = $('#gdlr-core-skin-style-temp');
							if( !temp_style.length ){
								temp_style = $('<style type="text/css" id="gdlr-core-skin-style-temp" ></style>');
								$('body').append(temp_style);
							}

							temp_style.html(data.style);
						}
					}else if( data.status == 'failed' ){
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
					}
				}
			});	
			
		},		
		
		// ajax get template
		ajax_get_template: function( template, options ){
			
			var t = this;
			var settings = $.extend({
				error: function(jqXHR, textStatus, errorThrown){
					gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });

					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					if( data.status == 'success' ){
						var new_content = $(data.content);
						template.replaceWith(new_content);
						
						t.pb.trigger('gdlr-core-element-change');
						t.update_undo_stack(true);
						
					}else if( data.status == 'failed' ){
						template.slideUp(200, function(){ $(this).remove() });
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});		
					}
				}
			}, options);
			
			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: settings.data,
				dataType: 'json',
				beforeSend: settings.beforeSend,
				error: settings.error,
				success: settings.success
			});	
			
		}, // ajax_get_template
		
		// ajax save template
		ajax_save_template: function( template, options ){
			
			var t = this;
			var settings = $.extend({
				error: function(jqXHR, textStatus, errorThrown){
					
					gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });

					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					
					if( data.status == 'success' ){
						gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});	
					}
					
					gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});	
					
				}
			}, options);
			
			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: settings.data,
				dataType: 'json',
				beforeSend: settings.beforeSend,
				error: settings.error,
				success: settings.success
			});	
			
		},				
		
		// update element visual
		update_element_preview: function( elem ){
			
			var t = this;
			
			elem.find('.gdlr-core-page-builder-item-container-preview').each(function(){
				var item = $(this).closest('.gdlr-core-page-builder-item');
				var preview_container = $(this).addClass('gdlr-core-now-loading');
				
				$.ajax({
					type: 'POST',
					url: page_builder_var.text.ajaxurl,
					data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_element_preview',
							'type': item.attr('data-type'), 'value': item.data('value'), 'post_id': page_builder_var.post_id },
					dataType: 'json',
					error: function(jqXHR, textStatus, errorThrown){
						
						// for displaying the debug text
						console.log(jqXHR, textStatus, errorThrown);
					},
					success: function(data){
						preview_container.removeClass('gdlr-core-now-loading');

						if( data.status == 'success' ){
							if( data.preview_content ){
								var content = $(data.preview_content);
								content.find('a').on('click', function(){
									return false;
								});
								//content.find('a').attr('href', '#gdlr-pb-link').removeClass('gdlr-core-ilightbox');

								preview_container.html(content);
								t.pb.trigger('gdlr-core-element-change');
								t.update_undo_stack(true);
							}

						}
					}
				});	
			});
		}, // update_element_preview
		
		// update wrapper visual
		update_wrapper_preview: function( elem ){
			
			var t = this;
			var wrapper_val = t.pb_update_tempalte_val(elem);

			elem.addClass('gdlr-core-now-loading');
			
			$.ajax({
				type: 'POST',
				url: page_builder_var.text.ajaxurl,
				data: { 'security': t.pb_nonce, 'action': 'gdlr_core_get_pb_wrapper_preview', 
						'type': elem.attr('data-type'), 'value': wrapper_val, 'post_id': page_builder_var.post_id },
				dataType: 'json',
				error: function(jqXHR, textStatus, errorThrown){
					
					// for displaying the debug text
					console.log(jqXHR, textStatus, errorThrown);
				},
				success: function(data){
					elem.removeClass('gdlr-core-now-loading');
		
					if( data.status == 'success' ){
						if( data.preview_content ){
							var new_content = $(data.preview_content);
							elem.replaceWith(new_content);
							t.sortable_column_order();
						}
					}
				}
			});	
			
		}, // update_element_preview		
		
		////////////////////////
		// lightbox template
		////////////////////////		
		
		lb_save_custom_template: function( value, type ){
			
			var t = this;
			var custom_template_lb = $(page_builder_var.template.custom_template_lb);
			
			$('body').append(custom_template_lb);
			custom_template_lb.css({opacity: 0}).animate({opacity: 1}, 200);
			
			// bind close button
			custom_template_lb.find('.gdlr-core-custom-template-lb-head-close').on('click', function(){
				custom_template_lb.animate({opacity: 0}, 200, function(){
					$(this).remove();
				});
			});
			
			// save template click
			custom_template_lb.find('.gdlr-core-custom-template-save').on('click', function(){
				var template_name = $(this).siblings('.gdlr-core-custom-template-name').val();
				
				if( template_name == '' ){
					gdlr_core_alert_box({
						status: 'failed',
						head: page_builder_var.template.custom_template_no_text_head, 
						message: page_builder_var.template.custom_template_no_text_message
					});	
				}else{
					custom_template_lb.animate({opacity: 0}, 200, function(){
						$(this).remove();
					});
					
					// save the template action
					$.ajax({
						type: 'POST',
						url: page_builder_var.text.ajaxurl,
						data: { 'security': t.pb_nonce, 'action': 'gdlr_core_save_pb_custom_template', 'value': value, 'type': type, 'title': template_name },
						dataType: 'json',
						error: function(jqXHR, textStatus, errorThrown){
							
							gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
							
							// for displaying the debug text
							console.log(jqXHR, textStatus, errorThrown);
						},
						success: function(data){

							if( data.status == 'success' ){
								var new_custom_template = $(data.nav_item);
								
								t.head_content.children('[data-content-type="custom-template"]').find('.gdlr-core-page-builder-head-content-custom-template-container').append(new_custom_template);
							}
							
							gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});
						}
					});					
					
				}
			});
			
		}, // lb_save_custom_template

		lb_save_sync_template: function( value, type, current_item ){
			
			var t = this;
			var sync_template_lb = $(page_builder_var.template.custom_template_lb);
			
			$('body').append(sync_template_lb);
			sync_template_lb.css({opacity: 0}).animate({opacity: 1}, 200);
			
			// bind close button
			sync_template_lb.find('.gdlr-core-custom-template-lb-head-close').on('click', function(){
				sync_template_lb.animate({opacity: 0}, 200, function(){
					$(this).remove();
				});
			});
			
			// save template click
			sync_template_lb.find('.gdlr-core-custom-template-save').on('click', function(){
				var template_name = $(this).siblings('.gdlr-core-custom-template-name').val();
				
				if( template_name == '' ){
					gdlr_core_alert_box({
						status: 'failed',
						head: page_builder_var.template.custom_template_no_text_head, 
						message: page_builder_var.template.custom_template_no_text_message
					});	
				}else{
					sync_template_lb.animate({opacity: 0}, 200, function(){
						$(this).remove();
					});
					
					// save the template action
					$.ajax({
						type: 'POST',
						url: page_builder_var.text.ajaxurl,
						data: { 'security': t.pb_nonce, 'action': 'gdlr_core_save_pb_sync_template', 'value': value, 'type': type, 'title': template_name },
						dataType: 'json',
						error: function(jqXHR, textStatus, errorThrown){
							
							gdlr_core_alert_box({ status: 'failed', head: page_builder_var.text.error_head, message: page_builder_var.text.error_message });
							
							// for displaying the debug text
							console.log(jqXHR, textStatus, errorThrown);
						},
						success: function(data){

							if( data.status == 'success' ){
								var new_custom_template = $(data.nav_item);
								t.head_content.children('[data-content-type="sync-template"]').find('.gdlr-core-page-builder-head-content-sync-template-container').append(new_custom_template);
								
								// add slug to current item
								if( data.sync_template_slug ){
									current_item.attr('data-sync-template', data.sync_template_slug);
								}
							}
							
							gdlr_core_alert_box({status: data.status, head: data.head, message: data.message});
						}
					});					
					
				}
			});
			
		}, // lb_save_custom_template
		
		////////////////////////
		// lightbox options
		////////////////////////
	
		lb_options: function( content, current_item ){
			
			var t = this;
			var lb_wrapper = $('<div class="gdlr-core-lightbox-content-wrapper"></div>');
			var lb_content = $('<div class="gdlr-core-lightbox-content"></div>');
			
			lb_wrapper.append(lb_content);
			lb_content.append(content);
			$('body').append(lb_wrapper).css({overflow: 'hidden', width: $('body').width()});
			lb_wrapper.css({opacity: 0}).animate({opacity: 1}, 400);

			// action for html option script
			var html_option = new gdlrCoreHtmlOption(lb_content);

			// bind skin settings
			lb_content.find('.gdlr-core-html-option-skin-edit').on('click', function(){
				var combobox = $(this).siblings('.gdlr-core-html-option-item-input').find('select');
				t.ajax_get_skin_options(combobox);
			});
			
			// action for tab
			var tab_head = lb_content.find('#gdlr-core-page-builder-options-tab-head');
			var tab_content = lb_content.find('#gdlr-core-page-builder-options-tab-content');
			tab_head.children('.gdlr-core-page-builder-options-tab-head-item').on('click', function(){
				if( $(this).hasClass('gdlr-core-active') ){ return; }
				
				var active_tab = $(this).attr('data-tab-slug');
				$(this).addClass('gdlr-core-active').siblings().removeClass('gdlr-core-active');
				tab_content.find('[data-tab-slug="' + active_tab + '"]').fadeIn(200).siblings().css('display', 'none');
			});
			
			// close tab action
			lb_content.find('#gdlr-core-page-builder-options-head-close').on('click', function(){
				lb_wrapper.fadeOut(200, function(){
					$(this).remove();
					$('body').css({overflow: '', width: ''});
				})
			});

			// save button 
			var lb_save_button = lb_content.find('#gdlr-core-page-builder-options-save');
			lb_save_button.on('click', function(){
				var new_value = html_option.get_val();
				current_item.data('value', new_value);

				lb_wrapper.fadeOut(200, function(){
					$(this).remove();
					$('body').css({overflow: '', width: ''});
				});

				t.update_undo_stack();

				if( current_item.is('.gdlr-core-page-builder-item') ){
					t.update_element_preview(current_item);
				}else if( current_item.is('.gdlr-core-page-builder-wrapper, .gdlr-core-page-builder-column, .gdlr-core-page-builder-flexbox') ){
					t.update_wrapper_preview(current_item);
				}
			});
			
			var mousedown_target = null;
			lb_wrapper.on('mousedown', function(e){
				// scroll bar
				if( lb_wrapper.outerWidth() < e.clientX + 20 ){
					mousedown_target = null;
				}else{
					mousedown_target = e.target;
				}
				
			});
			var mouseup_target = null;
			lb_wrapper.on('mouseup', function(e){
				if( lb_wrapper.outerWidth() < e.clientX + 20 ){
					mouseup_target = null;
				}else{
					mouseup_target = e.target;
					if( $(mouseup_target).is(lb_wrapper) && $(mousedown_target).is(mouseup_target)  ){
						lb_save_button.trigger('click');
					}
				}
				
			});

		}, 

		// lb_skin_options	
		lb_skin_options: function( content, combobox ){
			
			var t = this;
			var lb_wrapper = $('<div class="gdlr-core-lightbox-content-wrapper gdlr-core-lightbox-skin-content-wrapper"></div>');
			var lb_content = $('<div class="gdlr-core-lightbox-content"></div>');
			
			lb_wrapper.append(lb_content);
			lb_content.append(content);
			$('body').append(lb_wrapper);
			lb_wrapper.css({opacity: 0}).animate({opacity: 1}, 400);

			// action for html option script
			var html_option = new gdlrCoreHtmlOption(lb_content);
			
			// close option
			lb_content.find('#gdlr-core-page-builder-options-head-close').on('click', function(){
				lb_wrapper.fadeOut(200, function(){
					$(this).remove();
				})
			});

			// save button 
			var lb_save_button = lb_content.find('#gdlr-core-page-builder-options-save');
			lb_save_button.on('click', function(){
				var new_value = html_option.get_val();
				t.ajax_update_skin(new_value, combobox);

				lb_wrapper.fadeOut(200, function(){
					$(this).remove();
				});
			});
			
			var lb_content_clicked = false;
			lb_content.children().on('click', function(){
				lb_content_clicked = true;
			});
			lb_wrapper.on('click', function(){
				if( !lb_content_clicked ){
					lb_save_button.trigger('click');
				}
				lb_content_clicked = false;
			});

		}, // lb_options
		
		///////////////////
		// sortable
		///////////////////
		
		// bind the specify container
		bind_container_area: function( elem ){
			
			var t = this;

			new gdlrCoreCustomSortable(elem, {

				mousedown: function(e){
					// prevent dragging on live preview
					if( t.body_content.is('.gdlr-core-pb-livemode.gdlr-core-live-preview') ){
						return false;
					}

					return true;
				},
				startdragging: function(e){ 
					t.sortable_column_order();
					t.pb.addClass('gdlr-core-dragging')
					t.body_content.removeClass('gdlr-core-no-dragging');
				},
				dragging: function(e, current_item, placeholder){ 
					// check if the place holder is in new line
					placeholder.removeClass('gdlr-core-placeholder-clear');

					var prev_item = placeholder.prev();
					while( prev_item.length > 0 && prev_item.css('display') == 'none' ){
						prev_item = prev_item.prev();
					}

					if( prev_item.length > 0 ){
						if( prev_item.hasClass('gdlr-core-last-row') || (prev_item.hasClass('gdlr-core-last-item') && !current_item.is('.gdlr-core-page-builder-column, [data-type="column"]')) ){
							placeholder.addClass('gdlr-core-placeholder-clear');
						}
					}
				},
				updateposition: function(e){
					t.update_undo_stack();
				},
				enddragging: function(e){
					t.sortable_column_order();
						
					// put new content on same update stack
					t.update_undo_stack(true);

					t.pb.removeClass('gdlr-core-dragging');
					t.body_content.addClass('gdlr-core-no-dragging');

					t.pb.trigger('gdlr-core-element-resize');
					t.pb.trigger('gdlr-core-show-tooltip');
				}
			},

			// template
			{
				container: t.head_content,

				item_type: function(current_item){
					if(	current_item.attr('data-template') == 'wrapper' ){
						if( current_item.attr('data-type') == 'column' ){ 
							return 'column';
						}else if( current_item.attr('data-type') == 'column' ){
							return 'flexbox';
						}
						return 'wrapper';
					}else if( current_item.attr('data-template') == 'element' ){
						return 'item';
					}else{
						// for template & custom template
						return 'wrapper';
					}
				},
				template_item: function(current_item){
					return t.get_template(current_item);
				}
			});
			
		}, // bind_container_area
		
		sortable_column_order: function(){
			
			var size = 0;
			
			this.body_content.find('.gdlr-core-page-builder-column').removeClass('gdlr-core-last-row gdlr-core-last-item gdlr-core-first').each(function(){
				
				// ignore the starter element
				if( $(this).css('display') == 'none' ){ return; }
				
				// this is a first column if last item isn't a column
				var prev_item = $(this).prev();
				while( prev_item.css('display') == 'none' || prev_item.hasClass('gdlr-core-page-builder-placeholder') ){
					prev_item = prev_item.prev();
				}
				if( size == 0 || !prev_item.is('.gdlr-core-page-builder-column') ){
					$(this).addClass('gdlr-core-first')
					size = 0;
				}

				// added the column size
				size += parseInt($(this).attr('data-column'));
				if( size % 60 == 0 ){
					$(this).addClass('gdlr-core-last-row');
					size = 0;
				}else if( size > 60 ){
					$(this).addClass('gdlr-core-first');
					size = parseInt($(this).attr('data-column'))
				}
					
				// determine if it's last item	
				var next_item = $(this).next();
				while( next_item.length > 0 && (next_item.css('display') == 'none' || next_item.hasClass('gdlr-core-page-builder-placeholder')) ){
					next_item = next_item.next();
				}
				if( next_item.length == 0 || !next_item.is('.gdlr-core-page-builder-column') ){
					$(this).addClass('gdlr-core-last-item');
				}
			});	
			
		}, // sortable_column_order
		
	} // gdlrCorePageBuilder.prototype	
	
	$(document).on('ready', function(){
		
		new gdlrCorePageBuilder();
		
	}); // document ready

	$(window).on('load', function(){

		$('.edit-post-layout__content, .block-editor-editor-skeleton__content, .interface-interface-skeleton__content').on('scroll', function(){
			$(window).trigger('pb-scroll');
		});
	});
	
})(jQuery);

/*! jQuery Transit */
(function(t,e){if(typeof define==="function"&&define.amd){define(["jquery"],e)}else if(typeof exports==="object"){module.exports=e(require("jquery"))}else{e(t.jQuery)}})(this,function(t){t.transit={version:"0.9.12",propertyMap:{marginLeft:"margin",marginRight:"margin",marginBottom:"margin",marginTop:"margin",paddingLeft:"padding",paddingRight:"padding",paddingBottom:"padding",paddingTop:"padding"},enabled:true,useTransitionEnd:false};var e=document.createElement("div");var n={};function i(t){if(t in e.style)return t;var n=["Moz","Webkit","O","ms"];var i=t.charAt(0).toUpperCase()+t.substr(1);for(var r=0;r<n.length;++r){var s=n[r]+i;if(s in e.style){return s}}}function r(){e.style[n.transform]="";e.style[n.transform]="rotateY(90deg)";return e.style[n.transform]!==""}var s=navigator.userAgent.toLowerCase().indexOf("chrome")>-1;n.transition=i("transition");n.transitionDelay=i("transitionDelay");n.transform=i("transform");n.transformOrigin=i("transformOrigin");n.filter=i("Filter");n.transform3d=r();var a={transition:"transitionend",MozTransition:"transitionend",OTransition:"oTransitionEnd",WebkitTransition:"webkitTransitionEnd",msTransition:"MSTransitionEnd"};var o=n.transitionEnd=a[n.transition]||null;for(var u in n){if(n.hasOwnProperty(u)&&typeof t.support[u]==="undefined"){t.support[u]=n[u]}}e=null;t.cssEase={_default:"ease","in":"ease-in",out:"ease-out","in-out":"ease-in-out",snap:"cubic-bezier(0,1,.5,1)",easeInCubic:"cubic-bezier(.550,.055,.675,.190)",easeOutCubic:"cubic-bezier(.215,.61,.355,1)",easeInOutCubic:"cubic-bezier(.645,.045,.355,1)",easeInCirc:"cubic-bezier(.6,.04,.98,.335)",easeOutCirc:"cubic-bezier(.075,.82,.165,1)",easeInOutCirc:"cubic-bezier(.785,.135,.15,.86)",easeInExpo:"cubic-bezier(.95,.05,.795,.035)",easeOutExpo:"cubic-bezier(.19,1,.22,1)",easeInOutExpo:"cubic-bezier(1,0,0,1)",easeInQuad:"cubic-bezier(.55,.085,.68,.53)",easeOutQuad:"cubic-bezier(.25,.46,.45,.94)",easeInOutQuad:"cubic-bezier(.455,.03,.515,.955)",easeInQuart:"cubic-bezier(.895,.03,.685,.22)",easeOutQuart:"cubic-bezier(.165,.84,.44,1)",easeInOutQuart:"cubic-bezier(.77,0,.175,1)",easeInQuint:"cubic-bezier(.755,.05,.855,.06)",easeOutQuint:"cubic-bezier(.23,1,.32,1)",easeInOutQuint:"cubic-bezier(.86,0,.07,1)",easeInSine:"cubic-bezier(.47,0,.745,.715)",easeOutSine:"cubic-bezier(.39,.575,.565,1)",easeInOutSine:"cubic-bezier(.445,.05,.55,.95)",easeInBack:"cubic-bezier(.6,-.28,.735,.045)",easeOutBack:"cubic-bezier(.175, .885,.32,1.275)",easeInOutBack:"cubic-bezier(.68,-.55,.265,1.55)"};t.cssHooks["transit:transform"]={get:function(e){return t(e).data("transform")||new f},set:function(e,i){var r=i;if(!(r instanceof f)){r=new f(r)}if(n.transform==="WebkitTransform"&&!s){e.style[n.transform]=r.toString(true)}else{e.style[n.transform]=r.toString()}t(e).data("transform",r)}};t.cssHooks.transform={set:t.cssHooks["transit:transform"].set};t.cssHooks.filter={get:function(t){return t.style[n.filter]},set:function(t,e){t.style[n.filter]=e}};if(t.fn.jquery<"1.8"){t.cssHooks.transformOrigin={get:function(t){return t.style[n.transformOrigin]},set:function(t,e){t.style[n.transformOrigin]=e}};t.cssHooks.transition={get:function(t){return t.style[n.transition]},set:function(t,e){t.style[n.transition]=e}}}p("scale");p("scaleX");p("scaleY");p("translate");p("rotate");p("rotateX");p("rotateY");p("rotate3d");p("perspective");p("skewX");p("skewY");p("x",true);p("y",true);function f(t){if(typeof t==="string"){this.parse(t)}return this}f.prototype={setFromString:function(t,e){var n=typeof e==="string"?e.split(","):e.constructor===Array?e:[e];n.unshift(t);f.prototype.set.apply(this,n)},set:function(t){var e=Array.prototype.slice.apply(arguments,[1]);if(this.setter[t]){this.setter[t].apply(this,e)}else{this[t]=e.join(",")}},get:function(t){if(this.getter[t]){return this.getter[t].apply(this)}else{return this[t]||0}},setter:{rotate:function(t){this.rotate=b(t,"deg")},rotateX:function(t){this.rotateX=b(t,"deg")},rotateY:function(t){this.rotateY=b(t,"deg")},scale:function(t,e){if(e===undefined){e=t}this.scale=t+","+e},skewX:function(t){this.skewX=b(t,"deg")},skewY:function(t){this.skewY=b(t,"deg")},perspective:function(t){this.perspective=b(t,"px")},x:function(t){this.set("translate",t,null)},y:function(t){this.set("translate",null,t)},translate:function(t,e){if(this._translateX===undefined){this._translateX=0}if(this._translateY===undefined){this._translateY=0}if(t!==null&&t!==undefined){this._translateX=b(t,"px")}if(e!==null&&e!==undefined){this._translateY=b(e,"px")}this.translate=this._translateX+","+this._translateY}},getter:{x:function(){return this._translateX||0},y:function(){return this._translateY||0},scale:function(){var t=(this.scale||"1,1").split(",");if(t[0]){t[0]=parseFloat(t[0])}if(t[1]){t[1]=parseFloat(t[1])}return t[0]===t[1]?t[0]:t},rotate3d:function(){var t=(this.rotate3d||"0,0,0,0deg").split(",");for(var e=0;e<=3;++e){if(t[e]){t[e]=parseFloat(t[e])}}if(t[3]){t[3]=b(t[3],"deg")}return t}},parse:function(t){var e=this;t.replace(/([a-zA-Z0-9]+)\((.*?)\)/g,function(t,n,i){e.setFromString(n,i)})},toString:function(t){var e=[];for(var i in this){if(this.hasOwnProperty(i)){if(!n.transform3d&&(i==="rotateX"||i==="rotateY"||i==="perspective"||i==="transformOrigin")){continue}if(i[0]!=="_"){if(t&&i==="scale"){e.push(i+"3d("+this[i]+",1)")}else if(t&&i==="translate"){e.push(i+"3d("+this[i]+",0)")}else{e.push(i+"("+this[i]+")")}}}}return e.join(" ")}};function c(t,e,n){if(e===true){t.queue(n)}else if(e){t.queue(e,n)}else{t.each(function(){n.call(this)})}}function l(e){var i=[];t.each(e,function(e){e=t.camelCase(e);e=t.transit.propertyMap[e]||t.cssProps[e]||e;e=h(e);if(n[e])e=h(n[e]);if(t.inArray(e,i)===-1){i.push(e)}});return i}function d(e,n,i,r){var s=l(e);if(t.cssEase[i]){i=t.cssEase[i]}var a=""+y(n)+" "+i;if(parseInt(r,10)>0){a+=" "+y(r)}var o=[];t.each(s,function(t,e){o.push(e+" "+a)});return o.join(", ")}t.fn.transition=t.fn.transit=function(e,i,r,s){var a=this;var u=0;var f=true;var l=t.extend(true,{},e);if(typeof i==="function"){s=i;i=undefined}if(typeof i==="object"){r=i.easing;u=i.delay||0;f=typeof i.queue==="undefined"?true:i.queue;s=i.complete;i=i.duration}if(typeof r==="function"){s=r;r=undefined}if(typeof l.easing!=="undefined"){r=l.easing;delete l.easing}if(typeof l.duration!=="undefined"){i=l.duration;delete l.duration}if(typeof l.complete!=="undefined"){s=l.complete;delete l.complete}if(typeof l.queue!=="undefined"){f=l.queue;delete l.queue}if(typeof l.delay!=="undefined"){u=l.delay;delete l.delay}if(typeof i==="undefined"){i=t.fx.speeds._default}if(typeof r==="undefined"){r=t.cssEase._default}i=y(i);var p=d(l,i,r,u);var h=t.transit.enabled&&n.transition;var b=h?parseInt(i,10)+parseInt(u,10):0;if(b===0){var g=function(t){a.css(l);if(s){s.apply(a)}if(t){t()}};c(a,f,g);return a}var m={};var v=function(e){var i=false;var r=function(){if(i){a.unbind(o,r)}if(b>0){a.each(function(){this.style[n.transition]=m[this]||null})}if(typeof s==="function"){s.apply(a)}if(typeof e==="function"){e()}};if(b>0&&o&&t.transit.useTransitionEnd){i=true;a.bind(o,r)}else{window.setTimeout(r,b)}a.each(function(){if(b>0){this.style[n.transition]=p}t(this).css(l)})};var z=function(t){this.offsetWidth;v(t)};c(a,f,z);return this};function p(e,i){if(!i){t.cssNumber[e]=true}t.transit.propertyMap[e]=n.transform;t.cssHooks[e]={get:function(n){var i=t(n).css("transit:transform");return i.get(e)},set:function(n,i){var r=t(n).css("transit:transform");r.setFromString(e,i);t(n).css({"transit:transform":r})}}}function h(t){return t.replace(/([A-Z])/g,function(t){return"-"+t.toLowerCase()})}function b(t,e){if(typeof t==="string"&&!t.match(/^[\-0-9\.]+$/)){return t}else{return""+t+e}}function y(e){var n=e;if(typeof n==="string"&&!n.match(/^[\-0-9\.]+/)){n=t.fx.speeds[n]||t.fx.speeds._default}return b(n,"ms")}t.transit.getTransitionValue=d;return t});